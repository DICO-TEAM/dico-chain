# Know Your Customer (KYC)

## Interface

### Dispatchable Functions

#### For general users

* `set_kyc` - Set the associated KYC of an account; a small deposit is reserved if not already taken.
* `clear_kyc` - Remove an account's associated KYC of an account; the deposit is returned.
* `request_judgement` - Request a judgement from a IAS, paying a fee.
* `apply_certification` - apply certification

#### For identity authentication service(IAS)

* `ias_set_fee` - Set the fee required to be paid for a judgement to be given by the IAS.
* `ias_provide_judgement` - Provide a judgement to an KYC account.
* `ias_request_sword_holder` - Certification is handed over to sword holder



#### For sword holder

* `sword_holder_provide_judgement` - Provide a judgement to an kyc account.
* `sword_holder_set_fee` -  Set the fee required to be paid for a judgement to be given by the sword holder.


#### For sudo super-users(Sudo)
* `add_ias` - Add a new ias provider to the system. tips: Formed by election
* `add_sword_holder` - Add a new sword holder to the system. tips: Formed by election
* `kill_ias` - Forcibly remove the associated ias; the deposit is lost.
* `kill_sword_holder` - Forcibly remove the associated sword holder; the deposit is lost.
* `remove_kyc` - Forcibly remove kyc from kyc list and add to black list.
