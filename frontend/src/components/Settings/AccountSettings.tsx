import { useAuthStore } from "../../stores/authStore";

export function AccountSettings() {
  const { isAuthenticated, user, login, logout } = useAuthStore();

  if (!isAuthenticated) {
    return (
      <div className="p-6 max-w-md">
        <h2 className="text-xl font-bold mb-6 text-gray-900 dark:text-gray-100">
          Account
        </h2>
        <p className="text-sm text-gray-500 mb-4">
          Sign in to enable device sync and cloud features.
        </p>
        <div className="space-y-3">
          <button
            onClick={() => login("google")}
            className="w-full px-4 py-3 text-sm font-medium rounded-lg border border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-800 flex items-center justify-center gap-2"
          >
            Sign in with Google
          </button>
          <button
            onClick={() => login("apple")}
            className="w-full px-4 py-3 text-sm font-medium rounded-lg bg-black text-white hover:bg-gray-800 flex items-center justify-center gap-2"
          >
            Sign in with Apple
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="p-6 max-w-md">
      <h2 className="text-xl font-bold mb-6 text-gray-900 dark:text-gray-100">
        Account
      </h2>

      <div className="mb-6 p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
        <p className="text-sm font-medium text-gray-900 dark:text-gray-100">
          {user?.nickname}
        </p>
        <p className="text-xs text-gray-500">{user?.email}</p>
      </div>

      <button
        onClick={logout}
        className="px-4 py-2 text-sm rounded-lg bg-red-100 text-red-600 hover:bg-red-200"
      >
        Sign Out
      </button>
    </div>
  );
}
