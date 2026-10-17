# Feature: skills

A skill is a folder:

```
Skills/
└── summarize-email/
    ├── SKILL.md          # the prompt + a one-line description
    ├── tool.json         # JSON-schema for the tool call
    └── script.sh         # optional local script
```

The skill surface is read by the Ask page when the model wants to
call a tool. Local scripts run with the user's shell; prompts
stream into the system prompt of the chosen model.
