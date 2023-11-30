{% for snippet in snippets %}
# {{ snippet.complexity_name }}
    {% for snippet_data in snippet.snippets %}
        *complexity:* **{{ snippet_data.complexity }}**

        *start line:* **{{ snippet_data.start_line }}**

        *end line:* **{{ snippet_data.end_line }}**

        ```{{ language }}
        {{ snippet_data.text }}
        ```
    {% endfor %}
{% endfor %}

