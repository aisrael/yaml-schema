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

  Scenario: Tuple validation
    Given a YAML schema:
      ```
      type: array
      prefixItems:
        - type: number
        - type: string
        - enum:
          - Street
          - Avenue
          - Boulevard
        - enum:
          - NW
          - NE
          - SW
          - SE
      ```
    Then it should accept:
      ```
      - 1600
      - Pennsylvania
      - Avenue
      - NW
      ```
    # "Drive" is not one of the acceptable street types 
    But it should NOT accept:
      ```
      - 24
      - Sussex
      - Drive
      ```
    # This address is missing a street number
    And it should NOT accept:
      ```
      - Palais de l'Élysée
      ```
    # It's ok to not provide all of the items
    But it should accept:
      ```
      - 10
      - Downing
      - Street
      ```
    # And, by default, it's also ok to provide additional items
    And it should accept:
      ```
      - 1600
      - Pennsylvania
      - Avenue
      - NW
      - Washington
      ```
