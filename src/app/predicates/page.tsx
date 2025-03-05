"use client"
import React, { useState } from 'react';
import { BN, ScriptTransactionRequest, bn, Address, Output, OutputType } from 'fuels';
import { NftFixedPriceSwapPredicate } from './NftFixedPriceSwapPredicate';
import { useWallet } from '@fuels/react';

const PredicatePage: React.FC = () => {
    const { wallet } = useWallet();
    const [predicate, setPredicate] = useState<NftFixedPriceSwapPredicate | null>(null);
    const [config, setConfig] = useState<{ [key: string]: string }>({
        FEE_AMOUNT: '',
        FEE_ASSET: '',
        TREASURY_ADDRESS: '',
        ASK_AMOUNT: '',
        ASK_ASSET: '',
        RECEIVER: '',
        NFT_ASSET_ID: '',
    });

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        setConfig({
            ...config,
            [e.target.name]: e.target.value
        })
    }

    const initializePredicate = async () => {
        if (!wallet) return;

        const configurableConstants = {
            FEE_AMOUNT: bn(config.FEE_AMOUNT),
            FEE_ASSET: { bits: config.FEE_ASSET },
            TREASURY_ADDRESS: { bits: config.TREASURY_ADDRESS },
            ASK_AMOUNT: bn(config.ASK_AMOUNT),
            ASK_ASSET: { bits: config.ASK_ASSET },
            RECEIVER: { bits: config.RECEIVER },
            NFT_ASSET_ID: { bits: config.NFT_ASSET_ID },
        };

        const newPredicate = new NftFixedPriceSwapPredicate({
            provider: wallet.provider,
            data: [],
            configurableConstants,
        });

        try {
            console.log("Transferring NFT to Predicate Address...");

            const transferTx = await wallet.transfer(
                newPredicate.address,
                bn(1),
                config.NFT_ASSET_ID,
                { gasLimit: 100_000 }
            );

            await transferTx.waitForResult();
            console.log("NFT successfully transferred to Predicate.");

            //Inputs
            let predicateInputs = await newPredicate.getResourcesToSpend([
                { amount: bn(1), assetId: config.NFT_ASSET_ID },
            ]);

            let takerInputs = await wallet.getResourcesToSpend([
                { amount: bn(config.ASK_AMOUNT), assetId: config.ASK_ASSET },
            ]);

            //Outputs
            const outputToReceiver: Output = {
                type: OutputType.Coin,
                to: config.RECEIVER,
                amount: bn(config.ASK_AMOUNT),
                assetId: config.ASK_ASSET,
            };

            const outputToTreasury: Output = {
                type: OutputType.Coin,
                to: config.TREASURY_ADDRESS,
                amount: bn(config.FEE_AMOUNT),
                assetId: config.ASK_ASSET,
            };

            const outputToTaker: Output = {
                type: OutputType.Coin,
                to: wallet.address.toString(),
                amount: bn(1),
                assetId: config.NFT_ASSET_ID,
            };
          

            const transactionRequest = new ScriptTransactionRequest({
                gasLimit: bn(500_000),
                maxFee: bn(100_000),
            });

            transactionRequest.addResources([...predicateInputs, ...takerInputs]);
            transactionRequest.outputs.push(outputToReceiver, outputToTreasury, outputToTaker);

            console.log("Simulating Transaction...");
            console.log("Simulating Transaction...");
            try {
                await transactionRequest.estimateAndFund(newPredicate);
                console.log("Transaction estimated and funded successfully.");
            } catch (e) {
                console.error("Transaction estimation failed:", e);
            }
            console.log("Transaction Outputs:", transactionRequest.outputs);


            const simulateTx = await wallet.simulateTransaction(transactionRequest);
            console.log("Predicate Simulation Result:", simulateTx);

            setPredicate(newPredicate);
            console.log("Predicate Initialized:", newPredicate);
        } catch (error) {
            console.error("Error initializing predicate:", error);
        }
    };

    return (
        <div>
            <h1>Fuel Predicate Setup</h1>
            {Object.entries(config).map(([key, value]) => (
                <div key={key}>
                    <label>{key}:</label>
                    <input
                        type="text"
                        name={key}
                        value={value}
                        onChange={handleChange}
                    />
                </div>
            ))}
            <button onClick={initializePredicate}>Initialize Predicate</button>


        </div>
    );
};

export default PredicatePage;