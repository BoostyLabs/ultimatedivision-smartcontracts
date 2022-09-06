const Nft = artifacts.require("Nft");
const NftPresale = artifacts.require("NftPresale");
const NftSale = artifacts.require("NftSale");

module.exports = async(deployer, network, accounts) => {

  let proxyRegistryAddress = "";
  if (network === 'rinkeby') {
    proxyRegistryAddress = "0xf57b2c51ded3a29e6891aba85459d600256cf317";
  } else {
    proxyRegistryAddress = "0xa5409ec958c83c3f309868babaca7c86dcb077c1";
  }
  //pk - 5aefce0a2d473f59578fa7dee6a122d6509af1e0f79fcbee700dfcfeddabe4cc
  const verifyAddress = "0x4604F4045bb2b2d998dEd660081eb6ebC19C9f1e";
  // const verifyAddress = accounts[0];
  const name = "Ultimate Division";
  const symbol = "UD";
  const baseTokenURI = "https://ultimatedivision.com/nftdrop/founder/";
  const contractURI = "https://ultimatedivision.com/nftdrop/prgDescription.json";
  const operators = [accounts[0], accounts[1], accounts[2]];

  await deployer.deploy(Nft, proxyRegistryAddress, verifyAddress, name, symbol, contractURI, baseTokenURI, operators); 

  await deployer.deploy(NftSale, Nft.address, accounts[0]);

  let nftInstance = await Nft.deployed();
  await nftInstance.updateOperator(NftSale.address, true);
}
