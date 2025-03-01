"use client"
import type { Metadata } from "next";
import { FuelWalletConnector } from '@fuels/connectors';
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import { FuelProvider } from '@fuels/react';
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const queryClient = new QueryClient();

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
       <QueryClientProvider client={queryClient}>
      <FuelProvider 
      fuelConfig={
        {
          connectors: [new FuelWalletConnector()],
        }
      }
      >
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased`}
      >
      
        {children}
        
      </body>
      </FuelProvider>
      </QueryClientProvider>
    </html>
  );
}
