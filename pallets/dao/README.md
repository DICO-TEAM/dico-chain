# Overview
* The DAO module is only used to assist the ICO module, and every ICO of a project can constitute a DAO.
* The DAO only supports two proposals in the ICO module, one is whether to release funds for the project and the other is whether to terminate the project ICO.
* In the DAO proposal, the amount of each person participating in the project ICO is counted as the number of votes, which is also different from the Collective module.
***
## Interface
### Dispatchable Functions
* For users whom participate in ico
    * `propose` User makes a proposal
    * `vote`  User voted for the proposal
  ***
* For everyone
  * `close` User closes proposal
  ***
* For sudo super-users(Sudo)
  * `disapprove_proposal` Reject a proposal
