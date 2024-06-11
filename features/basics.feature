Feature: Basic YAML schema

  Scenario: Empty YAML
    Given a YAML schema:
      ```
      ```

    Then it should accept:
        ```
        42
        ```
    And it should accept:
        ```
        "I'm a string"
        ```
    And it should accept:
        ```
        an:
          - arbitrarily
          - nested
        data: structure
        ```
