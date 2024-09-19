Feature: Schema Composition

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

  Scenario: properties with oneOf ZZZ
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
            reviewers: true
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
