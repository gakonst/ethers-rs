# Intro
Welcome to the hands-on guide for the ethers-rs library!

This documentation contains a collection of examples demonstrating how to use the library to build Ethereum-based applications in Rust. The examples cover a range of topics, from basic smart contract interactions to more advanced usage of ethers-rs.

```admonish info 
You can find the official ethers-rs documentation on docs.rs - [here](https://docs.rs/ethers/0.5.0/ethers/).
```

Each example includes a detailed description of the functionality being demonstrated, as well as complete code snippets that you can use as a starting point for your own projects.

We hope that these docs will help you get started with ethers-rs and give you a better understanding of how to use the library to build your own web3 applications in Rust. If you have any questions or need further assistance, please don't hesitate to reach out to the ethers-rs community.

The following is a brief overview diagram of the  topis covered in this guide.

```mermaid
graph LR
  %% The code below is for styling the graph 
  %%-------------------------------------------------  
  %%{init: {'theme':'dark', 'themeVariables':{'textColor':' #ffffff ', 'nodeBorder':'#ff2d00', 'edgeLabelBackground':'#000000'  ,'lineColor':'#87ff00', 'fontSize':'14px', 'curve':'linear'}}}%%

  %%-------------------------------------------------
  %% Actual Diagram code is below
  
    A[Ethers-rs <br> Manual] --> A1[Providers]
	A --> A2[Middlewear]
    A --> A3[Contracts]
    A --> A4[Events]
    A --> A5[Subscriptions]
    A --> A6[Queries]
    A --> A7[Transactions]
    A --> A8[Wallets]
    A --> A9[Big numbers]
    A --> A10[Anvil]
```
```admonish bug 
This diagram is incomplete and will undergo continuous changes.
```
