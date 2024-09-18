Feature: Arrays

  Scenario: Array type
    Given a YAML schema:
      ```
      type: array
      ```
    Then it should accept:
      ```
      - 1
      - 2
      - 3
      - 4
      - 5
      ```
    And it should accept:
      ```
      - 3
      - different
      - types: "of values"
      ```
    But it should NOT accept:
      ```
      Not: "an array"
      ```

  Scenario: Array items
    Given a YAML schema:
      ```
      type: array
      items:
        type: number
      ```
    Then it should accept:
      ```
      - 1
      - 2
      - 3
      - 4
      - 5
      ```
    # A single non-number causes the entire array to be invalid
    But it should NOT accept:
      ```
      - 1
      - 2
      - "3"
      - 4
      - 5
      ```
    # The empty array is always valid
    And it should accept:
      ```
      []
      ```
