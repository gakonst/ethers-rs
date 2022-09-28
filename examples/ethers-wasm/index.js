const ethers = import('./pkg');

ethers
    .then(m => {
        m.deploy().catch(console.error);
    })
    .catch(console.error);
