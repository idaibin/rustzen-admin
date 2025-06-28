import useSWR from "swr";

// 根目录的占位符页面
const HomePage = () => {
  // 使用 SWR 请求后端用户列表接口
  const { data, error, isLoading } = useSWR("/system/users");

  if (error) return <div>Failed to load</div>;
  if (isLoading) return <div>Loading...</div>;

  return (
    <div>
      <h1 className="text-4xl font-bold text-center text-gray-800 dark:text-white">
        Home
      </h1>
      <p className="mt-4 text-center">Data from backend:</p>
      <pre className="mt-2 p-4 bg-gray-100 dark:bg-gray-800 rounded-md">
        {JSON.stringify(data, null, 2)}
      </pre>
    </div>
  );
};

export default HomePage;
