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
