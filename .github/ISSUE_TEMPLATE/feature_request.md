name: Feature request
description: Suggest a new capability for one of the parsers
labels: [enhancement]
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
    id: problem
    attributes:
      label: Problem
      description: What problem would this feature solve?
    validations:
      required: true
  - type: textarea
    id: proposal
    attributes:
      label: Proposed solution
      description: How should it work? Consider MSRV, no_std, and zero/low allocation constraints.
    validations:
      required: false
