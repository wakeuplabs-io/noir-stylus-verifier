import { ZkAccount, ACCOUNT_MESSAGE } from "@voting/core";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useSignMessage } from "wagmi";
import { toHex } from "viem";

class QueryKeyFactory {
  static zkAccount = () => ["zk-account"];
}

export const useZkAccount = () => {
  return useQuery({
    queryKey: QueryKeyFactory.zkAccount(),
    queryFn: async () => {
      const zkAccount = window.localStorage.getItem("zk-account");
      if (!zkAccount) {
        return { address: null, secret: null, privateKey: null };
      }

      return JSON.parse(zkAccount) as {
        address: `0x${string}`;
        secret: `0x${string}` ;
        privateKey: `0x${string}`;
      };
    },
    initialData: { address: null, secret: null, privateKey: null },
    refetchInterval: false,
  });
};

export const useZkConnect = () => {
  const queryClient = useQueryClient();
  const { signMessageAsync } = useSignMessage();
  return useMutation({
    mutationFn: async () => {
      const signedMessage = await signMessageAsync({
        message: ACCOUNT_MESSAGE,
      });
      const zkAccount = await ZkAccount.buildFromSignature(signedMessage);

      const account = {
        address: toHex(zkAccount.address),
        secret: toHex(zkAccount.secret),
        privateKey: toHex(zkAccount.privateKey),
      };

      window.localStorage.setItem("zk-account", JSON.stringify(account));

      return account;
    },
    onSuccess: (result) => {
      queryClient.setQueryData(QueryKeyFactory.zkAccount(), result);
    },
  });
};

export const useZkDisconnect = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async () => {
      window.localStorage.removeItem("zk-account");
      queryClient.setQueryData(QueryKeyFactory.zkAccount(), {
        address: null,
        secret: null,
        privateKey: null,
      });
    },
  });
};
