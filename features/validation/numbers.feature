Feature: Numeric types

  Scenario: integer ZZZ
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

  Scenario: Multiples
    Given a YAML schema:
      ```
      type: number
      multipleOf: 10
      ```
    Then it should accept:
      ```
      0
      ```
    And it should accept:
      ```
      10
      ```
    And it should accept:
      ```
      20
      ```
    But it should NOT accept:
      ```
      23
      ```
