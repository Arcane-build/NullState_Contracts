"use client";

import React, {useState}from 'react';
import {ScriptTransactionRequest} from "fuels";
import { useFuel, useIsConnected } from '@fuels/react';


const Home = () => {
    const { fuel } = useFuel();
    const { isConnected } = useIsConnected();
    const [address, setAddress] = useState<string | null>(null);
    
    const handleConnect = async()=>{
        if(!fuel){
            console.error("Fuel not connected");
            return;
        }
        try {
            await fuel.connect();
            const accounts = await fuel.accounts();
            if(accounts.length>0){
                const address = accounts[0];
                const sliceAddress = address.slice(0, 6) + "..." + address.slice(-4);
                setAddress(sliceAddress);
            }
        } catch (error) {
            console.error("Error connecting wallet", error);
        }
    }
   
    return (
        <div>
          <button onClick={handleConnect} style={{
          backgroundColor: "green",
          color: "black",
          padding: "0.75rem 1.25rem",
          border: "none",
          borderRadius: "6px",
          cursor: "pointer",
        }}>
            {isConnected && address ? address : "Connect Wallet"}
        </button>
        </div>
    );
};

export default Home;