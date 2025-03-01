"use client"
import React, { useState } from 'react';
import { BN } from 'fuels';
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
            FEE_AMOUNT: new BN(config.FEE_AMOUNT),
            FEE_ASSET: { bits: config.FEE_ASSET },
            TREASURY_ADDRESS: { bits: config.TREASURY_ADDRESS },
            ASK_AMOUNT: new BN(config.ASK_AMOUNT),
            ASK_ASSET: { bits: config.ASK_ASSET },
            RECEIVER: { bits: config.RECEIVER },
            NFT_ASSET_ID: { bits: config.NFT_ASSET_ID },
        };

        const newPredicate = new NftFixedPriceSwapPredicate({
            provider: wallet.provider,
            data: [],
            configurableConstants,
        });
     
        setPredicate(newPredicate);
        console.log('Predicate Initialized:', newPredicate);
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