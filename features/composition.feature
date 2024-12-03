Feature: Schema Composition

  Scenario: anyOf
    Given a YAML schema:
      ```
      anyOf:
        - type: string
          maxLength: 5
        - type: number
          minimum: 0
      ```
    Then it should accept:
      ```
      "short"
      ```
    But it should NOT accept:
      ```
      "too long"
      ```
    And it should accept:
      ```
      12
      ```
    But it should NOT accept:
      ```
      -5
      ```

  Scenario: oneOf
    Given a YAML schema:
      ```
      oneOf:
        - type: number
          multipleOf: 5
        - type: number
          multipleOf: 3
      ```
    Then it should accept:
      ```
      10
      ```
    And it should accept:
      ```
      9
      ```
    But it should NOT accept:
      ```
      2
      ```
    # Multiple of _both_ 5 and 3 is rejected
    And it should NOT accept:
      ```
      15
      ```

  Scenario: oneOf null or object
    Given a YAML schema:
      ```
      type: object
      properties:
        child:
          oneOf:
            - type: null
            - type: object
              properties:
                name:
                  type: string
              required:
                - name
      additionalProperties: false
      ```
    Then it should accept:
      ```
      child: null
      ```
    And it should accept:
      ```
      child:
        name: John
      ```
    But it should NOT accept:
      ```
      name: John
      ```

  Scenario: properties with oneOf
    Given a YAML schema:
      ```
      type: object
      properties:
        name:
          type: string
        github:
          type: object
          properties:
            environments:
              type: object
              patternProperties:
                "^[a-zA-Z][a-zA-Z0-9_-]*$":
                  type: object
                  properties:
                    reviewers:
                      oneOf:
                        - type: null
                        - type: array
                          items:
                            type: string
      ```
    Then it should accept:
      ```
      name: test
      github:
        environments:
          development:
            reviewers: null
      ```
    And it should accept:
      ```
      name: test
      github:
        environments:
          production:
            reviewers:
              - alice
              - bob
      ```
    But it should NOT accept:
      ```
      name: test
      github:
        environments:
          development:
            reviewers: true # true is not one of the acceptable values
      ```

  Scenario: patternProperties with oneOf
    Given a YAML schema:
      ```
      type: object
      patternProperties:
        ^[a-zA-Z0-9]+$:
          oneOf:
            - type: null
            - type: object
              properties:
                name:
                  type: string
      ```
    Then it should accept:
      ```
      a1b:
        name: John
      ```

  Scenario: not
    Given a YAML schema:
      ```
      not:
        type: string
      ```
    Then it should accept:
      ```
      42
      ```
    And it should accept:
      ```
      key: value
      ```
    But it should NOT accept:
      ```
      "I am a string"
      ```
