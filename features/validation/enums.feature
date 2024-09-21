Feature: Enumerated values

  @enum
  Scenario: Enumerated values
    Given a YAML schema:
      ```
      enum:
        - red
        - amber
        - green
      ```
    Then it should accept:
      ```
      red
      ```
    And it should accept:
      ```
      green
      ```
    But it should NOT accept:
      ```
      blue      
      ```

  @enum
  Scenario: enum without a type
    Given a YAML schema:
      ```
      enum:
        - red
        - amber
        - green
        - null
        - 42
      ```
    Then it should accept:
      ```
      red
      ```
    And it should accept:
      ```
      null
      ```
    And it should accept:
      ```
      42
      ```
    But it should NOT accept:
      ```
      0
      ```
