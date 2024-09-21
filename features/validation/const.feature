Feature: Constant values

  @const
  Scenario: const
    Given a YAML schema:
      ```
      type: object
      properties:
        country:
          const: United States of America
      ```
    Then it should accept:
      ```
      country: United States of America
      ```
    But it should NOT accept:
      ```
      country: Canada
      ```
