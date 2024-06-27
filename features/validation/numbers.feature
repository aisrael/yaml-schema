Feature: Numeric types

  Scenario: integer
    Given a YAML schema:
      ```
      type: integer
      ```
    Then it should accept:
      ```
      42
      ```
    And it should accept:
      ```
      -1
      ```
    And it should accept:
      ```
      1.0
      ```
    But it should NOT accept:
      ```
      3.1415926
      ```
    And it should NOT accept:
      ```
      "42"
      ```
