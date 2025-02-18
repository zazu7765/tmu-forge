# tmu-forge
building your degree, the way you want.

## Overview
TMU Forge is a tool designed to help students at Toronto Metropolitan University plan their degree.
Initially, the tool is targetted towards students from the Faculty of Science, and focusing on core electives.

### Goals:
- Provide visualizations for students to plan their electives
- Provide a way to save progress and share with others
- Allow uploads of unofficial transcripts that can be parsed to determine completed courses
- Helped students make informed decisions about course selection

### Non-Goals:
- Replace the official MyServiceHub Academic Advisement Report Tool
- Provide official degree requirements / academic advice
- Handle course registration/enrollment
- Maintain real-time course information (e.g exam schedules, section counts, etc.)

## Structure
```
tmu-forge/ (Forge Suite)
├── crucible/          # Scraper
├── anvil/             # CLI tool
├── foundry-api/       # Backend
├── foundry/           # Frontend
└── hearth/            # Shared resources
```
### Crucible
**Crucible** is a web scraper that fetches course information from the official TMU course calendar.

### Anvil
**Anvil** is a command-line tool used to interact with **crucible**, and performs additional tasks such as

### Foundry
**Foundry** is the frontend of TMU Forge, and is where students can interact with the tool.

### Hearth
Shared resources between the different components of TMU Forge.
