{% for username, opts in args.users -%}
## User: `{{ opts.name }}`

Username: {{ username }}
Home: `{{ opts.home }}`
Groups: {% for group in opts.groups %}{% if loop.index > 1 %}, {% endif %}`{{ group }}`{% else %}None{% endfor %}

{% endfor -%}
