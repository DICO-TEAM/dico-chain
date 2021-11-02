# ICO
## Overview
The ICO module is a platform for the project party to conduct ICO.
Its characteristic is to prevent the project party from doing evil and give all users a fair opportunity to participate.
***
## Interface
### Dispatchable Functions
* For general users
    * `join` User participation ico.
    * `user_release_ico_amount` The user releases the amount of pledged participation in the ico.
    * `unlock` User unlock funds(Part of the amount locked after release)
    * `get_reward` users receive rewards after the ico ends.

* For sudo super-users(Sudo)
    * `set_system_ico_amount_bound` Set the minimum and maximum amount that all users can participate in ico.

* For project party
    * `initiate_ico` The project party initiated ico.
    * `request_release` The project party applies for the release of funds.
    * `initiator_set_ico_amount_bound` The project party sets the maximum and minimum amount of participation in ico.
    * `initiator_set_ico_max_times` The project party sets the maximum number of times users can participate in ico.
    * `user_release_ico_amount` The project party releases the amount of pledged participation in the ico.
* For DICO foundation
    * `permit_ico` The foundation agrees to the project party to initiate ico.
    * `reject_ico` The foundation refuses the project party to initiate an ico.
* For DAO
    * `terminate_ico` DAO forced to terminate ico halfway.
    * `permit_release` DAO agrees to the request of the project party to release funds.


