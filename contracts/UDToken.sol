// SPDX-License-Identifier: MIT

pragma solidity =0.8.0;

import 'openzeppelin-solidity/contracts/token/ERC20/ERC20.sol';
import 'openzeppelin-solidity/contracts/utils/math/SafeMath.sol';
import 'openzeppelin-solidity/contracts/token/ERC20/presets/ERC20PresetMinterPauser.sol';
import 'openzeppelin-solidity/contracts/access/Ownable.sol';
import 'openzeppelin-solidity/contracts/token/ERC20/utils/SafeERC20.sol';



// File: openzeppelin-solidity/contracts/ownership/CanReclaimToken.sol

/**
 * @title Contracts that should be able to recover tokens
 * @author SylTi
 * @dev This allow a contract to recover any ERC20 token received in a contract by transferring the balance to the contract owner.
 * This will prevent any accidental loss of tokens.
 */
contract CanReclaimToken is Ownable {
  using SafeERC20 for ERC20;

  /**
   * @dev Reclaim all ERC20Basic compatible tokens
   * @param _token ERC20Basic The address of the token contract
   */
  function reclaimToken(ERC20 _token) external onlyOwner {
      uint256 balance = _token.balanceOf(address(this));
      _token.safeTransfer(owner(), balance);
  }
  
}

contract UDToken is ERC20PresetMinterPauser("Ultimate Division token", "UDT"), Ownable, CanReclaimToken {

    //@todo change it
	//pk - 5aefce0a2d473f59578fa7dee6a122d6509af1e0f79fcbee700dfcfeddabe4cc
	address public verifyAddress = 0x4604F4045bb2b2d998dEd660081eb6ebC19C9f1e;

    mapping (address => uint) private claimNonce;


    function burn(uint value) public override onlyOwner {
        super.burn(value);
    }

    function claim(uint value, uint nonce, bytes calldata _signature) public
    {
        require(verify(_signature, value, nonce), "invalid signature");
        require(claimNonce[msg.sender] == nonce, "invalid nonce");
        claimNonce[msg.sender] += 1;
        _mint(msg.sender, value);
    }
    
    function decimals() public view virtual override returns (uint8) {
        return 18;
    }

    function finishMinting() public view onlyOwner returns (bool) {
        return false;
    }

    function renounceOwnership() public view override onlyOwner {
        revert("renouncing ownership is blocked");
    }

    /// signature methods.
	function verify(bytes memory _signature, uint value, uint nonce) internal view returns(bool) {
		bytes32 message = prefixed(keccak256(abi.encodePacked(msg.sender, address(this), value, nonce)));
        return (recoverSigner(message, _signature) == verifyAddress);
	}

    function recoverSigner(bytes32 message, bytes memory sig)
        internal
        pure
        returns (address)
    {
        (uint8 v, bytes32 r, bytes32 s) = abi.decode(sig, (uint8, bytes32, bytes32));

        return ecrecover(message, v, r, s);
    }

    /// builds a prefixed hash to mimic the behavior of eth_sign.
    function prefixed(bytes32 hash) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", hash));
    }
}
