import Dashboard from "./features/dashboard/components/Dashboard";
import Sidebar from "./features/sidebar/components/Sidebar";

function App() {
  return (
    <main className="h-screen p-6 flex justify-between">
      <Sidebar />
      <Dashboard />
    </main>
  );
}

export default App;
