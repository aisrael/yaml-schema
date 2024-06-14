Feature: CLI usage

  Scenario: Display the version
    When the following command is run:
      ```
      ys version
      ```
    Then it should exit with status code 0
    And it should output:
      ```
      ys 0.1.0
      ```
