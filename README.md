# 🚀 nullstate NFT Marketplace

## Overview  
This project is a fully on-chain, non-custodial NFT marketplace built on the [Fuel Blockchain](https://fuel.network/).  
Using Fuel’s predicate functionality, this marketplace enables secure peer-to-peer NFT trading without requiring any intermediary custody of user assets. Buyers and sellers interact through smart conditions, ensuring that NFTs and funds only move when conditions are met — all trustlessly.  

## Key Features  
- **Non-Custodial**: Assets never leave the user’s control unless conditions are fulfilled.  
- **Predicate-Based Trades**: Listings are powered by Fuel predicates, enabling conditional execution.  
- **Decentralized**: Fully on-chain listing, bidding, and settlement logic.  
- **Secure**: No centralized contract holds user funds or NFTs.  
- **Fast & Low-Cost**: Built on Fuel's high-throughput modular execution layer.  

## How It Works  
1. **Listing an NFT**:  
   - The seller creates a predicate that encodes the conditions for the NFT sale (price, NFT details).  
   - The NFT is transferred to the predicate address, not a centralized contract.  

2. **Buying an NFT**:  
   - The buyer fulfills the predicate condition by submitting a transaction with the required funds.  
   - If the conditions are satisfied, the NFT is transferred to the buyer, and funds are released to the seller — all in one atomic transaction.  

3. **Canceling a Listing**:  
   - The seller can withdraw the NFT from the predicate at any time if it hasn’t been purchased, retaining full control.  

## Acknowledgments  
- Built on top of [Fuel Labs](https://fuel.network)  
- Inspired by the power of UTXO-based smart contracts and predicates  
