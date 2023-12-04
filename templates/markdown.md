{%- for complexity in snippets -%}
# {{ complexity }}
{% for data in snippets[complexity] %}
## Snippet {{ loop.index }}

*complexity value:* **{{ data.complexity }}**

*start line:* **{{ data.start_line }}**

*end line:* **{{ data.end_line }}**

```{{ language }}
{{ data.text }}
```
{% endfor %}
{%- endfor -%}
