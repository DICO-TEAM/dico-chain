## 1. Introduction
With the daos project, and with very little code, we completed the creation of the VC DAO template and gained all the governance capabilities that daos provides for the DAO. The VC DAO is a bit like CultDAO here, but more fully functional and flexible than CultDAO,
This is due to the excellent design of daos. In the kico network, we support the free creation of multiple assets. Each creator can create a VC DAO based on the assets he creates.
Users who own the asset are all members of the DAO. All fees from the asset transfer transaction go into the DAO's account, and the DAO members jointly decide on the use of the asset. Of course, this use is not just an ICO-related operation.
It also includes all transactions that ordinary users can perform. Regardless of the internal workings of the VC DAO, you can equate the VC DAO with a regular user, which is designed to give the DAO all the on-chain rights that a regular user has.

> Here are just a few examples to teach you about the create-dao, sudo, agency, square, and doas modules in daos. Although different DAOs may be somewhat different depending on the DAO template, they are generally the same.
In fact, the transactions that daos creates DAOS can perform are virtually unlimited and cannot be enumerated. The point here is to teach you how to use sudo, agency, or square to execute external transactions. 
If you want to learn more about the external transactions supported by the VC DAO and the corresponding call ids, 
Please look here code [https://github.com/DICO-TEAM/dico-chain/blob/main/runtime/tico/src/vc.rs#L30](https://github.com/DICO-TEAM/dico- chain/blob/main/runtime/tico/src/vc.rs#L30)
## 2. Run testnet
If you want to test daos, you should first run a test network locally.
Follow [these steps](https://github.com/DICO-TEAM/dico-chain#development) to launch your test net
## 3. Create VC DAO
As a developer, you can create your DAO template for people who have something in common.
In the VC DAO, we create DAO templates for groups of people with the same assets, hoping that they can decide together to do something. So, if you want to create a VC DAO, you must first create an asset.
Here Alice creates an asset with id 100 and the amount is 10000000000000000. Note that according to the requirements of the kico network, you must set metadata for your asset here, otherwise the transfer cannot be made.
![create asset](./vc-dao-pic/create%20asset.png)
Information about the asset created by Alice.
![asset info](./vc-dao-pic/asset%20info.png)
Alice's account balance under the asset.
![Alice asset info](./vc-dao-pic/Alice%20asset%20info.png)
Alice creates the VC DAO based on this asset. According to the requirements of the VC DAO template, only Alice can create the VC DAO for this asset, and no one else can.
As a developer, you are free to make rules in your DAO template about who can create a DAO.
![create dao](./vc-dao-pic/create%20dao.png)
DAO information.
![dao info](./vc-dao-pic/dao%20info.png)
We know that in kico networks, the execution of all transactions must be kico as gas, so we should give the account of the DAO ` 5 ep3dfmpjbgtnn3rt3skazkx6qehhnwvpe2juert7ijzg5ym ` transfer kico.
![transfer kico to DAO account](./vc-dao-pic/transfer%20kico%20to%20DAO%20account.png)
KICO balance in DAO account.
![DAO account kico info](./vc-dao-pic/DAO%20account%20kico%20info.png)
According to the VC DAO template, users with the same assets are the DAO members. So Alice should transfer asset 100 to Bob, Charlie, and Dave, making them DAO members.
As a developer, you are free to make rules in your DAO template about who qualifies as a DAO member.
Alice transfer asset to Bob.
![transfer asset to Bob](./vc-dao-pic/transfer%20asset%20to%20Bob.png)
Bob asset info.
![Bob asset info](./vc-dao-pic/Bob%20asset%20info.png)
Alice transfer asset to Charlie.
![transfer asset to Charlie](./vc-dao-pic/transfer%20asset%20to%20Charlie.png)
Charlie asset info.
![Charlie asset info](./vc-dao-pic/Charlie%20asset%20info.png)
Alice transfer asset to Dave.
![transfer asset to Dave](./vc-dao-pic/transfer%20asset%20to%20Dave.png)
Dave asset info.
![Dave asset info](./vc-dao-pic/Dave%20asset%20info.png)
The balance of Alice's remaining asset.
![Alice asset info_1](./vc-dao-pic/Alice%20asset%20info_1.png)
Above, we successfully created a VC DAO based on asset 100 according to the requirements of the VC DAO template.
## 4. sudo completes the initialization of the VC DAO
> It is possible to perform all external transactions that the DAO template allows, and the following are just examples.
* Set all agency members in the VC DAO
We know that each agency has its own members in daos. daos does not provide members selection rules. As a developer, you are free to give your Agency whatever members selection rules it wants. In the VC DAO,
The members of agency are assigned to suqare or sudo to set. Here, we directly use sudo to set the members.
![sudo set agency members](./vc-dao-pic/sudo%20set%20agency%20members.png)
agency members
![agency members info](./vc-dao-pic/agency%20members%20info.png)
* Set period parameters for the square module
    * sudo set the referendum cycle to 25 blocks
  ![sudo set launch period](./vc-dao-pic/sudo%20set%20launch%20period.png)
    * sudo sets the voting duration to 25 blocks
  ![sudo set voting period](./vc-dao-pic/sudo%20set%20voting%20period.png)
    * sudo sets the delay period to 25 blocks
  ![sudo set enactment period](./vc-dao-pic/sudo%20set%20enactment%20period.png)
* sudo sets Agency Origin for external transactions
In the agency module, all external transactions cannot be executed by default, only if Origin is set through sudo or square. So let's take transfer as an example,
We set the Origin for the external transaction, the transfer.
> As we know, in the VC DAO, we agreed that the transfer call id is 902. If you want to set Origin for another external transaction, you must first know the call id of that transaction in the DAO

![sudo set agency origin for transfer](./vc-dao-pic/sudo%20set%20agency%20origin%20for%20transfer.png)
Origin of the transfer in the Agency module.
![transfer func agency origin](./vc-dao-pic/transfer%20func%20agency%20origin.png)
* sudo sets square Origin for external transactions
In square, the vote for an external transaction is valid only if its (No *convition + Yes *convition) weight is greater than minweight. 
The default value is 0, which is set for each external transaction as needed. Again, let's take transfer as an example.
> As we know, in the VC DAO, we agreed that the transfer call id is 902. If you want to set Origin for another external transaction, you must first know the call id of that transaction in the DAO

![sudo set square origin for transfer](./vc-dao-pic/sudo%20set%20square%20origin%20for%20transfer.png)
Origin of the transfer in the square module.
![transfer func square origin](./vc-dao-pic/transfer%20func%20square%20origin.png)
## 5. Close sudo to complete the decentralization of DAO
close sudo
![close sudo](./vc-dao-pic/close%20sudo.png)
We find that the sudo account is None, completing decentralization.
![sudo account info](./vc-dao-pic/sudo%20account%20info.png)
## 6.Agency executes external transactions
In the above steps, we have set up agency Origin for the external transaction of transfer. Next, we transfer the tokens in DAO to Eve through agency.
Any external transaction executed by the agency is carried out by DAO account. Therefore, to transfer the assets in DAO to other accounts, 
the balance of such assets must be ensured in DAO account. Here we transfer 1000000000000000 from Alice's account to DAO's account.
Note that this is only done as a test; in fact, the balance of the DAO account in the VC DAO should come from the user's transfer fee.
Alice transfer asset to DAO account
![Alice transfer asset to DAO account](./vc-dao-pic/Alice%20transfer%20asset%20to%20DAO%20account.png)
DAO account asset info
![DAO account asset info](./vc-dao-pic/DAO%20account%20asset%20info.png)

Let's transfer the asset in the DAO to Eve 500000000000000.
Note that this module can directly execute or submit the proposal only if the external transaction of agency Origin is set up. 
For the transfer method, Origin requires a ratio of at least 1/2,
So our threshold here should be (4*1/2 = 2).
Now let's start to submit the transfer proposal.
![agency propose for transfer](./vc-dao-pic/agency%20propose%20for%20transfer.png)
proposal hash
![agency transfer proposal hash](./vc-dao-pic/agency%20transfer%20proposal%20hash.png)
proposal info
![agency transfer proposal info](./vc-dao-pic/agency%20transfer%20proposal%20info.png)
vote
![agency vote for transfer](./vc-dao-pic/agency%20vote%20for%20transfer.png)
vote result
![agency vote result](./vc-dao-pic/agency%20vote%20result.png)
Based on the vote result, we know that the vote has passed, so we can now close it.
![agency close proposal for transfer](./vc-dao-pic/agency%20close%20proposal%20for%20transfer.png)
DAO account asset info
![DAO account asset info_1](./vc-dao-pic/DAO%20account%20asset%20info_1.png)
Eve asset info
![Eve asset info](./vc-dao-pic/Eve%20asset%20info.png)
agency successfully executed a transfer transaction.
## 7. square executes external transactions
> It is possible to perform all external transactions that the DAO template allows, and the following are just examples

Let's transfer the asset in the DAO to Ferdie 200000000000000.

Let's submit a proposal first.
![square propose for transfer](./vc-dao-pic/square%20propose%20for%20transfer.png)
second. 
![Bob second](./vc-dao-pic/Bob%20second.png)
If the referendum can begin, choose the proposal with the highest amount as the new referendum.
![open table](./vc-dao-pic/open%20table.png)
referendum info
![referendum info](./vc-dao-pic/referendum%20info.png)
Bob went to vote for the referendum. Only asset 100 is supported for voting. This rule is also set by the developer of the VC DAO template. 
As a DAO template developer, you can design rules for what objects to use for square voting.
![Bob vote for referrendum](./vc-dao-pic/Bob%20vote%20for%20referrendum.png)
The total weight of the vote is greater than the minimum required weight, and the voting time is over.
![close proposal](./vc-dao-pic/close%20proposal.png)
DAO account asset info
![DAO account asset info_2](./vc-dao-pic/DAO%20account%20asset%20info_2.png)
Ferdie asset info
![Ferdie asset info](./vc-dao-pic/Ferdie%20asset%20info.png)
square successfully executed a transfer transaction