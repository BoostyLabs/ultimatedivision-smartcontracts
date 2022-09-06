const { catchRevert, catchRevertMessage} = require("./exceptions.js");
const Web3 = require("web3")
var abi = require('ethereumjs-abi')

const {
  BN,           // Big Number support
  constants,    // Common constants, like the zero address and largest integers
  time,
  balance
} = require('@openzeppelin/test-helpers');

const Nft = artifacts.require("Nft");
const NftPresale = artifacts.require("NftPresale");
const NftSale = artifacts.require("NftSale");

const createPassword = (userAddress, contractAddress) => {
  const hash = `0x${abi
    .soliditySHA3(['address', 'address'], [userAddress, contractAddress])
    .toString('hex')}`;
  const sig = web3.eth.accounts.sign(hash, "5aefce0a2d473f59578fa7dee6a122d6509af1e0f79fcbee700dfcfeddabe4cc");
  return web3.eth.abi.encodeParameters(
    ['uint8', 'bytes32', 'bytes32'],
    [sig.v, sig.r, sig.s],
  );
};

contract("Tests", accounts => {
  
  it('catch instances', async() => {
    nftInstance = await Nft.deployed();
    nftPresaleInstance = await NftPresale.deployed();
    nftSaleInstance = await NftSale.deployed();
  })

  it('check initial sets', async() => {
    let baseTokenURI = await nftInstance.baseTokenURI();
    assert.equal(baseTokenURI, 'https://ultimatedivision.com/nftdrop/founder/');

    //check operators
    assert.ok(await nftInstance.operators(accounts[1])); //true
    assert.ok(await nftInstance.operators(accounts[2])); //true
    assert.ok(!(await nftInstance.operators(accounts[3]))); //false
    assert.ok(await nftInstance.operators(nftPresaleInstance.address)); //true
    assert.ok(await nftInstance.operators(nftSaleInstance.address)); //true
  })

  it('check external functions', async() => {
    await nftInstance.updateBaseURI("https://nft-test-bucker.s3.eu-central-1.amazonaws.com/assets1/");
    let baseTokenURI = await nftInstance.baseTokenURI();
    assert.equal(baseTokenURI, 'https://nft-test-bucker.s3.eu-central-1.amazonaws.com/assets1/');

    await catchRevertMessage(nftInstance.mint(accounts[0], 1, {from: accounts[3]}), "only operators");
    await catchRevertMessage(nftInstance.mintBatch(accounts[0], [1, 2, 3], {from: accounts[3]}), "only operators");
    await catchRevertMessage(nftInstance.updateBaseURI("https://nft-test-bucker.s3.eu-central-1.amazonaws.com/assets2/", {from: accounts[2]}), "Ownable: caller is not the owner");
  })

  it('current node time should be less than startTime', async() => {
    let startTime = await nftPresaleInstance.START_TIME();
    let currentTime = await time.latest()
    assert.ok(currentTime < startTime);
  })
})