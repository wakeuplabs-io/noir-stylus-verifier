import { ZkAccount, ACCOUNT_MESSAGE } from "@voting/core";
import { useAccount, useSignMessage } from "wagmi";
import { toHex } from "viem";
import { useCallback, useEffect, useState } from "react";
import { createContext, useContext, type ReactNode } from "react";

interface ZkAccountContextType {
  account: ZkAccountDetails | null;
  connect: () => Promise<ZkAccountDetails>;
  disconnect: () => Promise<void>;
  isPending: boolean;
}

const ZkAccountContext = createContext<ZkAccountContextType | undefined>(undefined);

interface ZkAccountProviderProps {
  children: ReactNode;
}

export type ZkAccountDetails = {
  address: `0x${string}`;
  secret: `0x${string}`;
  privateKey: `0x${string}`;
};

// Local storage key for the ZK account
const ZK_ACCOUNT_KEY = "zk-account";

export const ZkAccountProvider = ({ children }: ZkAccountProviderProps) => {
  const { address: evmAddress } = useAccount();
  const { signMessageAsync } = useSignMessage();

  const [account, setAccount] = useState<ZkAccountDetails | null>(null);
  const [isPending, setIsPending] = useState(false);

  const recoverAccount = useCallback(async () => {
    const account = window.localStorage.getItem(ZK_ACCOUNT_KEY);
    if (!account) {
      return null;
    }
    return JSON.parse(account) as ZkAccountDetails;
  }, []);

  const saveAccount = useCallback(async (account: ZkAccountDetails) => {
    window.localStorage.setItem(ZK_ACCOUNT_KEY, JSON.stringify(account));
    setAccount(account);
  }, []);

  const connect = useCallback(async () => {
    setIsPending(true);

    try {
      if (!evmAddress) {
        throw new Error("No EVM account connected");
      }

      const signedMessage = await signMessageAsync({
        message: ACCOUNT_MESSAGE,
      });
      const zkAccount = await ZkAccount.buildFromSignature(signedMessage);

      const account: ZkAccountDetails = {
        address: toHex(zkAccount.address),
        secret: toHex(zkAccount.secret),
        privateKey: toHex(zkAccount.privateKey),
      };

      saveAccount(account);
      return account;
    } catch (error) {
      throw error;
    } finally {
      setIsPending(false);
    }
  }, [evmAddress, signMessageAsync]);

  const disconnect = useCallback(async () => {
    window.localStorage.removeItem(ZK_ACCOUNT_KEY);
    setAccount(null);
  }, []);

  useEffect(() => {
    recoverAccount().then(setAccount);
  }, [recoverAccount]);

  return (
    <ZkAccountContext.Provider
      value={{
        account,
        connect,
        disconnect,
        isPending,
      }}
    >
      {children}
    </ZkAccountContext.Provider>
  );
};

export const useZkAccount = () => {
  const context = useContext(ZkAccountContext);
  if (context === undefined) {
    throw new Error("useAccountContext must be used within an AccountProvider");
  }
  return context;
};
