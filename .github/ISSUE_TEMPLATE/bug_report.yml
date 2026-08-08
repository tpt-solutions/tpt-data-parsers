name: Bug report
description: Report a problem with one of the parsers
labels: [bug]
body:
  - type: dropdown
    id: crate
    attributes:
      label: Which crate?
      options:
        - tpt-jsonl-stream
        - tpt-geo-geojson
        - tpt-logfmt-parse
        - tpt-cron-parse
        - tpt-mime-pure
        - workspace / other
    validations:
      required: true
  - type: textarea
    id: what
    attributes:
      label: What happened?
      description: A clear description of the bug and the expected behavior.
    validations:
      required: true
  - type: textarea
    id: repro
    attributes:
      label: Reproduction
      description: Minimal input and code that triggers the behavior.
      render: rust
    validations:
      required: true
  - type: input
    id: version
    attributes:
      label: Crate version
      description: e.g. 0.1.0, or commit hash
    validations:
      required: false
