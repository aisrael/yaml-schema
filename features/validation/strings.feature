Feature: String validation

  Scenario: "type: string" should accept strings
    Given a YAML schema:
      ```
      type: string
      ```
    Then it should accept:
      ```
      "Déjà vu"
      ```
    And it should accept:
      ```
      ""
      ```
    And it should accept:
      ```
      "42"
      ```
    But it should NOT accept:
      ```
      42
      ```

  Scenario: length validation
    Given a YAML schema:
      ```
      type: string
      minLength: 2
      maxLength: 3
      ```
    Then it should NOT accept:
      ```
      "A"
      ```
    But it should accept:
      ```
      "AB"
      ```
    And it should accept:
      ```
      "ABC"
      ```
    But it should NOT accept:
      ```
      "ABCD"
      ```
