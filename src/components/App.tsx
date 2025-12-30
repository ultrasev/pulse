import { useState } from 'react';
import SystemDashboard from './SystemDashboard';
import IpInfo from './IpInfo';
import ProductPrice from './ProductPrice';

type Tab = 'system' | 'ipinfo' | 'price';

export default function App() {
    const [activeTab, setActiveTab] = useState<Tab>('system');

    return (
        <div className="h-full flex flex-col p-4 gap-4">
            <header className="flex justify-between items-center">
                <h1 className="text-2xl font-bold">System Monitor</h1>
                <div className="text-xs text-gray-500">v0.0.1</div>
            </header>

            {/* 标签导航 */}
            <nav className="flex gap-2 p-1 bg-gray-100 dark:bg-gray-800 rounded-lg">
                <button
                    onClick={() => setActiveTab('system')}
                    className={`flex-1 py-2 px-4 rounded-md font-medium transition-all ${
                        activeTab === 'system'
                            ? 'bg-white dark:bg-gray-700 shadow-sm text-blue-600 dark:text-blue-400'
                            : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'
                    }`}
                >
                    📊 System
                </button>
                <button
                    onClick={() => setActiveTab('ipinfo')}
                    className={`flex-1 py-2 px-4 rounded-md font-medium transition-all ${
                        activeTab === 'ipinfo'
                            ? 'bg-white dark:bg-gray-700 shadow-sm text-cyan-600 dark:text-cyan-400'
                            : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'
                    }`}
                >
                    🌐 IP Info
                </button>
                <button
                    onClick={() => setActiveTab('price')}
                    className={`flex-1 py-2 px-4 rounded-md font-medium transition-all ${
                        activeTab === 'price'
                            ? 'bg-white dark:bg-gray-700 shadow-sm text-orange-600 dark:text-orange-400'
                            : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'
                    }`}
                >
                    🏷️ Price
                </button>
            </nav>

            {/* 内容区域 */}
            {activeTab === 'system' && <SystemDashboard />}
            {activeTab === 'ipinfo' && <IpInfo />}
            {activeTab === 'price' && <ProductPrice />}
        </div>
    );
}
