const ethers = import('./pkg');

ethers
    .then(m => {
        m.setup();
        m.deploy().catch(console.error);
    })
    .catch(console.error);