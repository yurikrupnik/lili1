import axios, { AxiosResponse } from 'axios';
import { HttpAgentRequest } from './types.js';

export class HttpAgentClient {
  private timeout: number = 30000;

  async callAgent(request: HttpAgentRequest): Promise<any> {
    const { endpoint, method, payload, headers = {}, streaming = false } = request;

    if (streaming) {
      console.log('payload', payload);
      return this.streamCall(endpoint, method, payload, headers);
    }

    try {
      const response: AxiosResponse = await axios({
        url: endpoint,
        method,
        data: payload,
        headers: {
          'Content-Type': 'application/json',
          ...headers,
        },
        timeout: this.timeout,
      });

      return response.data;
    } catch (error) {
      if (axios.isAxiosError(error)) {
        throw new Error(`HTTP Agent call failed: ${error.response?.status} ${error.response?.statusText}`);
      }
      throw error;
    }
  }

  async streamCall(endpoint: string, method: string, payload: any, headers: Record<string, string>) {
    try {
      const response = await axios({
        url: endpoint,
        method,
        data: payload,
        headers: {
          'Content-Type': 'application/json',
          'Accept': 'text/event-stream',
          ...headers,
        },
        responseType: 'stream',
        timeout: this.timeout,
      });

      return new Promise((resolve, reject) => {
        let result = '';

        response.data.on('data', (chunk: Buffer) => {
          const data = chunk.toString();
          result += data;
        });

        response.data.on('end', () => {
          resolve(result);
        });

        response.data.on('error', (error: Error) => {
          reject(error);
        });
      });
    } catch (error) {
      if (axios.isAxiosError(error)) {
        throw new Error(`HTTP Stream call failed: ${error.response?.status} ${error.response?.statusText}`);
      }
      throw error;
    }
  }

  setTimeout(timeout: number) {
    this.timeout = timeout;
  }
}
