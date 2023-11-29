# {{ complexity_name }}

{% for snippet in snippets %}
*complexity:* **{{ snippet.complexity }}**

*start line:* **{{ snippet.start_line }}**

*end line:* **{{ snippet.end_line }}**

```{{ language }}
{{ snippet.text }}
```"
{% endfor %}
