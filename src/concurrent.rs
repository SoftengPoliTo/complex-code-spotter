use std::path::PathBuf;

use crossbeam::channel::{bounded, Receiver, Sender};
use crossbeam::scope;
use globset::GlobSet;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use walkdir::{DirEntry, WalkDir};

use crate::error::Error::Concurrent;
use crate::{Complexity, Result, Snippets};

type ProcFilesFunction = dyn Fn(PathBuf, &[(Complexity, usize)]) -> Option<Snippets> + Send + Sync;

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

/// Data related to files.
pub(crate) struct FilesData {
    /// Kind of files included in a search.
    pub include: GlobSet,
    /// Kind of files excluded from a search.
    pub exclude: GlobSet,
    /// File path.
    pub path: PathBuf,
}

/// A runner to process files concurrently.
pub(crate) struct ConcurrentRunner {
    proc_files: Box<ProcFilesFunction>,
    num_jobs: usize,
    complexities: Vec<(Complexity, usize)>,
}

impl ConcurrentRunner {
    /// Creates a new `ConcurrentRunner`.
    ///
    /// * `proc_files` - Function that processes each file found during
    ///    the search.
    /// * `num_jobs` - Number of jobs utilized to process files concurrently.
    pub(crate) fn new<ProcFiles>(
        proc_files: ProcFiles,
        num_jobs: usize,
        complexities: Vec<(Complexity, usize)>,
    ) -> Self
    where
        ProcFiles: 'static + Fn(PathBuf, &[(Complexity, usize)]) -> Option<Snippets> + Send + Sync,
    {
        Self {
            proc_files: Box::new(proc_files),
            num_jobs,
            complexities,
        }
    }

    fn send_file(&self, path: PathBuf, sender: &Sender<PathBuf>) -> Result<()> {
        sender
            .send(path)
            .map_err(|e| Concurrent(format!("Sender: {}", e).into()))
    }

    fn producer(&self, sender: Sender<PathBuf>, files_data: FilesData) -> Result<()> {
        let FilesData {
            path,
            ref include,
            ref exclude,
        } = files_data;

        if !path.exists() {
            return Err(Concurrent(
                format!("Sender: {:?} does not exist", path).into(),
            ));
        }
        if path.is_dir() {
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_entry(|e| !is_hidden(e))
            {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(e) => return Err(Concurrent(format!("Sender: {}", e).into())),
                };
                let path = entry.path().to_path_buf();
                if (include.is_empty() || include.is_match(&path))
                    && (exclude.is_empty() || !exclude.is_match(&path))
                    && path.is_file()
                {
                    self.send_file(path, &sender)?;
                }
            }
        } else if (include.is_empty() || include.is_match(&path))
            && (exclude.is_empty() || !exclude.is_match(&path))
            && path.is_file()
        {
            self.send_file(path, &sender)?;
        }

        Ok(())
    }

    fn consumer(&self, receiver: Receiver<PathBuf>, sender: Sender<Snippets>) -> Result<()> {
        // Extracts the snippets from the files received from the producer
        // and sends them to the composer.
        while let Ok(file) = receiver.recv() {
            if let Some(snippets) = (self.proc_files)(file.clone(), &self.complexities) {
                sender
                    .send(snippets)
                    .map_err(|e| Concurrent(format!("Sender: {}", e).into()))?;
            }
        }

        Ok(())
    }

    fn composer(&self, receiver: Receiver<Snippets>) -> Result<Vec<Snippets>> {
        let mut snippets_result = Vec::new();

        // Collects the snippets received from the consumer.
        while let Ok(snippets) = receiver.recv() {
            snippets_result.push(snippets)
        }

        Ok(snippets_result)
    }

    /// Runs the producer-consumer-composer approach to process the files
    /// contained in a directory and in its own subdirectories.
    ///
    /// * `files_data` - Information about the files to be included or excluded
    ///    from a search more the number of paths considered in the search.
    pub(crate) fn run(self, files_data: FilesData) -> Result<Vec<Snippets>>
    where
        Self: Sync,
    {
        let (producer_sender, consumer_receiver) = bounded(self.num_jobs);
        let (consumer_sender, composer_receiver) = bounded(self.num_jobs);

        let result = scope(|scope| {
            // Producer.
            scope.spawn(|_| self.producer(producer_sender, files_data));

            // Composer.
            let composer = scope.spawn(|_| self.composer(composer_receiver));

            // Consumer.
            (0..self.num_jobs).into_par_iter().try_for_each(|_| {
                self.consumer(consumer_receiver.clone(), consumer_sender.clone())
            })?;
            drop(consumer_sender);

            // Result produced by the composer.
            composer.join()?
        }); 
        
        match result {
            Ok(output) => output,
            Err(e) => Err(e.into()),
        }
    }
}
