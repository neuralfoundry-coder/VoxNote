import { create } from "zustand";

interface UserProfile {
  id: string;
  email: string;
  nickname: string;
  avatar_url: string | null;
}

interface AuthState {
  isAuthenticated: boolean;
  user: UserProfile | null;
  accessToken: string | null;

  login: (provider: "google" | "apple") => Promise<void>;
  logout: () => void;
  setUser: (user: UserProfile) => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  isAuthenticated: false,
  user: null,
  accessToken: null,

  login: async (_provider) => {
    // TODO: OAuth2 OIDC 플로우 실행
    set({
      isAuthenticated: true,
      user: {
        id: "user-1",
        email: "user@example.com",
        nickname: "VoxNote User",
        avatar_url: null,
      },
      accessToken: "mock_token",
    });
  },

  logout: () => {
    set({ isAuthenticated: false, user: null, accessToken: null });
  },

  setUser: (user) => set({ user }),
}));
