// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./ERC721Tradable.sol";
import "./INFT.sol";

contract Nft is INFT, ERC721Tradable {

	uint public lastTokenId = 10000;
	
	uint public lastTokenClaimed = 0;
	
	string public contractURI;
	string internal _baseTokenURI;

	mapping(address => bool) public operators;
	mapping(address => uint) public allowedToMint;
	mapping(address => uint) public minted;

	constructor(
		address _proxyRegistryAddress,
		string memory _name,
		string memory _symbol,
		string memory _contractURI,
		string memory __baseTokenURI,
		address[] memory _operators
	) ERC721Tradable(_name, _symbol, _proxyRegistryAddress)
	{
		contractURI = _contractURI;
		_baseTokenURI = __baseTokenURI;

		for(uint i = 0; i < _operators.length; i++) {
			operators[_operators[i]] = true;
		}
	}
	
	function baseTokenURI() public view override returns(string memory) {
	    // write base URI here
	    // https://api.lympo.io/pools/assets/62 - without token index
        return _baseTokenURI;
	}
	
	function _baseURI() internal view override returns(string memory) {
	    // write base URI here
	    // https://api.lympo.io/pools/assets/62 - without token index
        return _baseTokenURI;
    }

    function updateBaseURI(string memory __baseURI) public onlyOwner {
    	_baseTokenURI = __baseURI;
    }

	function updateOperator(address _operatorAddress, bool _status) public onlyOwner {
		operators[_operatorAddress] = _status;
	}
	
	function mint(address _to, uint _tokenId) public override {
		require(_tokenId > 0 && _tokenId <= lastTokenId, "token is not exists");
		require(operators[msg.sender], "only operators");
		_safeMint(_to, _tokenId);
	}

	function mintBatch(address _to, uint[] memory _tokenIds) public override {
		require(operators[msg.sender], "only operators");
		for(uint i = 0; i < _tokenIds.length; i++) {
			require(_tokenIds[i] > 0 && _tokenIds[i] <= lastTokenId, "token is not exists");
			_safeMint(_to, _tokenIds[i]);
		}
	}
	
	function mint(address _to) public override {
	    uint _tokenId = lastTokenClaimed + 1;
	    while (_exists(_tokenId)) {
	        _tokenId++;    
	    }
		require(_tokenId <= lastTokenId, "token is not exists");
		require(operators[msg.sender], "only operators");
		lastTokenClaimed = _tokenId;
		_safeMint(_to, _tokenId);
	}
	
	function mintBatch(address _to, uint amount) public override {
	    uint _tokenId = lastTokenClaimed + 1;
		require(operators[msg.sender], "only operators");
		require(amount > 0, "cannot claim 0 tokens");
		for(uint i = 0; i < amount; i++) {
		    while (_exists(_tokenId)) {
	            _tokenId++;    
	        }
			require(_tokenId <= lastTokenId, "token is not exists");
			_safeMint(_to, _tokenId);
		}
		lastTokenClaimed = _tokenId;
	}
	
	function checkLastTokenClaimedID(uint _tokenId) public returns (uint) {
	    uint _lastTokenClaimed = lastTokenClaimed;
	    require(_tokenId > _lastTokenClaimed, "already checked");
	    require(_tokenId <= lastTokenId, "token is not exists");
	    while (_exists(_lastTokenClaimed) && _lastTokenClaimed < _tokenId) {
	        _lastTokenClaimed++;    
	    }
	    lastTokenClaimed = _lastTokenClaimed;
	    return _lastTokenClaimed;
	}
}
