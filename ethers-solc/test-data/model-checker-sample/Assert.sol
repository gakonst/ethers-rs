contract Assert {
	function f(uint x) public pure {
		assert(x > 0);
	}
}
