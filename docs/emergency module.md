## 1. Introduction
With the daos project, and with very little code, we completed the creation of the VC DAO template and gained all the governance capabilities that daos provides for the DAO. The VC DAO is a bit like CultDAO here, but more fully functional and flexible than CultDAO,
This is due to the excellent design of daos. In the kico network, we support the free creation of multiple assets. Each creator can create a VC DAO based on the assets he creates.
Users who own the asset are all members of the DAO. All fees from the asset transfer transaction go into the DAO's account, and the DAO members jointly decide on the use of the asset. Of course, this use is not just an ICO-related operation.
It also includes all transactions that ordinary users can perform. Regardless of the internal workings of the VC DAO, you can equate the VC DAO with a regular user, which is designed to give the DAO all the on-chain rights that a regular user has.

> Here are just a few examples to teach you about the create-dao, sudo, agency, square, and doas modules in daos. Although different DAOs may be somewhat different depending on the DAO template, they are generally the same.
In fact, the transactions that daos creates DAOS can perform are virtually unlimited and cannot be enumerated. The point here is to teach you how to use sudo, agency, or square to execute external transactions.
If you want to learn more about the external transactions supported by the VC DAO and the corresponding call ids,
Please look here code [https://github.com/DICO-TEAM/dico-chain/blob/main/runtime/tico/src/vc.rs#L30](https://github.com/DICO-TEAM/dico-chain/blob/main/runtime/tico/src/vc.rs#L30)
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

Set Bob as an Emergency member
![set member](./vc-dao-pic/set%20bob%20as%20member.png)
Emergency members
![emergency members](./vc-dao-pic/emergency%20members.png)
set pledge
![set pledge](./vc-dao-pic/set%20pledge.png)
pledge amount
![pledge amount](./vc-dao-pic/pledge%20amount.png)
We know that in the square module, VotingPeriod cannot be 0, if it is 0, DAO cannot run normally, so our emergency is to solve this kind of problem. Now we assume that VotingPeriod is 0, and then we need to set it to 100 here, so that DAO can run normally again.
Let's start with an internal proposal
![internal track](./vc-dao-pic/internal%20track.png)
Internal proposal information and hash
![internal proposal info and proposal hash](./vc-dao-pic/internal%20track%20info%20and%20hash.png)

Here, we can wait for the block height to be greater than the end block, and then enact the proposal directly.
We don't do that here. We will show how to enact the proposal when the external proposal is made. For now,
we are temporarily rejecting internal proposals
![reject internal proposal](./vc-dao-pic/reject%20internal%20proposal.png)
We will find that the information and hash of the internal proposal no longer exist, indicating that the proposal has been successfully rejected
Now we make an external proposal for VotingPeriod
![external track](./vc-dao-pic/external%20track.png)
The hash and information of the external proposal
![internal proposal info and proposal hash](./vc-dao-pic/external%20proposal%20info%20and%20hash.png)
Here we can use members to reject external proposals, which will not be done here. Note that only members can reject external proposals.
Now we wait for the block height to be greater than the end block, then enact proposal
![enact proposal](./vc-dao-pic/enact%20proposal.png)
We see that VotingPeriod has been modified successfully. After passing the emergency processing of the emergency module, let the DAO run again.
![voting period](./vc-dao-pic/voting%20period.png)










