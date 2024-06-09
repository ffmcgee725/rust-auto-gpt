import axios from "axios";
import { useState } from "react";

type ForexPricePostRequest = {
  id: number;
  name: string;
  value: number;
};

type ForexPriceGetResponse = {
  id: number;
  name: string;
  value: number;
}[];

type ForexPriceGetByIdResponse = {
  id: number;
  name: string;
  value: number;
};

type ForexPricePutRequest = {
  id: number;
  name: string;
  value: number;
};

type RegisterRequest = {
  id: number;
  username: string;
  password: string;
};

type LoginRequest = {
  id: number;
  username: string;
  password: string;
};

type UseCallReturn = {
  postForexPrice: (requestBody: ForexPricePostRequest) => Promise<void>;
  getAllForexPrices: () => Promise<ForexPriceGetResponse>;
  getForexPriceById: (id: number) => Promise<ForexPriceGetByIdResponse>;
  updateForexPrice: (
    id: number,
    requestBody: ForexPricePutRequest
  ) => Promise<void>;
  register: (requestBody: RegisterRequest) => Promise<void>;
  login: (requestBody: LoginRequest) => Promise<void>;
  loading: boolean;
  error: any;
};

export const useCall = (): UseCallReturn => {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<any>(null);

  const postForexPrice = async (
    requestBody: ForexPricePostRequest
  ): Promise<void> => {
    setLoading(true);
    try {
      await axios.post("http://localhost:8080/forex_price", requestBody);
      setLoading(false);
    } catch (e) {
      setError(e as any);
      setLoading(false);
    }
  };

  const getAllForexPrices = async (): Promise<ForexPriceGetResponse> => {
    setLoading(true);
    try {
      const response = await axios.get("http://localhost:8080/forex_price");
      setLoading(false);
      return response.data as ForexPriceGetResponse;
    } catch (e) {
      setError(e as any);
      setLoading(false);
      return [];
    }
  };

  const getForexPriceById = async (
    id: number
  ): Promise<ForexPriceGetByIdResponse> => {
    setLoading(true);
    try {
      const response = await axios.get(
        `http://localhost:8080/forex_price/${id}`
      );
      setLoading(false);
      return response.data as ForexPriceGetByIdResponse;
    } catch (e) {
      setError(e as any);
      setLoading(false);
      return {} as ForexPriceGetByIdResponse;
    }
  };

  const updateForexPrice = async (
    id: number,
    requestBody: ForexPricePutRequest
  ): Promise<void> => {
    setLoading(true);
    try {
      await axios.put(`http://localhost:8080/forex_price/${id}`, requestBody);
      setLoading(false);
    } catch (e) {
      setError(e as any);
      setLoading(false);
    }
  };

  const register = async (requestBody: RegisterRequest): Promise<void> => {
    setLoading(true);
    try {
      await axios.post("http://localhost:8080/register", requestBody);
      setLoading(false);
    } catch (e) {
      setError(e as any);
      setLoading(false);
    }
  };

  const login = async (requestBody: LoginRequest): Promise<void> => {
    setLoading(true);
    try {
      await axios.post("http://localhost:8080/login", requestBody);
      setLoading(false);
    } catch (e) {
      setError(e as any);
      setLoading(false);
    }
  };

  return {
    postForexPrice,
    getAllForexPrices,
    getForexPriceById,
    updateForexPrice,
    register,
    login,
    loading,
    error,
  };
};
