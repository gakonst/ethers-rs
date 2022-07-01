pragma solidity >=0.6.0;
pragma experimental ABIEncoderV2;

contract MultiInputOutput {
	function dupeInt(uint256 input) public pure returns (uint256 outOne, uint256 outTwo) {
		return (input, input);
	}
	function arrayRelayer(uint256[] memory inputs) public pure returns (uint256[] memory outputs, uint someNumber) {
		outputs = new uint[](inputs.length);
		for(uint256 i = 0; i < inputs.length; i++) {
			outputs[i] = inputs[i];
		}
		someNumber = 42;
	}
	function singleUnnamed() public pure returns (uint) {
		return 0x45;
	}
	function callWithoutReturnData(uint256 input) public pure {
		// silence unused errors
		uint nothing = input;
		input = nothing;
		return;
	}
}
