import {
  CheckCircleOutlined,
  ClockCircleOutlined,
  ExclamationCircleOutlined,
  TeamOutlined,
  UserOutlined,
} from '@ant-design/icons';
import { Column, Line } from '@ant-design/plots';
import { Card, Progress, Statistic } from 'antd';
import useSWR from 'swr';

import { dashboardAPI } from '@/api/dashboard';
import { calculatePercent, convertUnit } from '@/util';

export default function DashboardPage() {
  return (
    <div className="flex flex-col gap-4">
      {/* 统计卡片 */}
      <StatsCard />

      {/* 系统健康状态 */}
      <div className="grid grid-cols-2 gap-4">
        <HealthCard />
        <MetricsCard />
      </div>

      {/* 用户活动趋势 */}
      <UserActivityTrendCard />
    </div>
  );
}

// 统计卡片
const StatsCard = () => {
  const { data: stats } = useSWR<Dashboard.Stats>(dashboardAPI.statsUrl);
  return (
    <div className="grid grid-cols-4 gap-4">
      <Card>
        <Statistic
          title="Total users"
          value={stats?.totalUsers}
          prefix={<UserOutlined />}
          valueStyle={{ color: '#3f8600' }}
        />
      </Card>
      <Card>
        <Statistic
          title="Active users"
          value={stats?.activeUsers}
          prefix={<TeamOutlined />}
          valueStyle={{ color: '#1890ff' }}
        />
      </Card>
      <Card>
        <Statistic
          title="Today logins"
          value={stats?.todayLogins}
          prefix={<ClockCircleOutlined />}
          valueStyle={{ color: '#722ed1' }}
        />
      </Card>

      <Card>
        <Statistic
          title="System uptime"
          value={stats?.systemUptime}
          prefix={<CheckCircleOutlined />}
          valueStyle={{ color: '#52c41a' }}
        />
      </Card>
    </div>
  );
};

// 系统健康状态
const HealthCard = () => {
  const { data: health } = useSWR<Dashboard.SystemHealth>(dashboardAPI.healthUrl);
  const memoryUsage = calculatePercent(health?.memoryUsed, health?.memoryTotal);
  const cpuUsage = calculatePercent(health?.cpuUsed, health?.cpuTotal);
  const diskUsage = calculatePercent(health?.diskUsed, health?.diskTotal);

  return (
    <Card title="System health status" extra={<ExclamationCircleOutlined />}>
      <div className="flex flex-col gap-5">
        <div>
          <div className="mb-2 flex justify-between">
            <span>Memory usage</span>
            <span>
              {convertUnit(health?.memoryUsed)} / {convertUnit(health?.memoryTotal)}
            </span>
          </div>
          <Progress percent={memoryUsage} status={memoryUsage > 80 ? 'exception' : 'normal'} />
        </div>
        <div>
          <div className="mb-2 flex justify-between">
            <span>CPU usage</span>
            <span>
              {health?.cpuUsed.toFixed(1)} / {health?.cpuTotal}
            </span>
          </div>
          <Progress percent={cpuUsage} status={cpuUsage > 80 ? 'exception' : 'normal'} />
        </div>
        <div>
          <div className="mb-2 flex justify-between">
            <span>Disk usage</span>
            <span>
              {convertUnit(health?.diskUsed)} / {convertUnit(health?.diskTotal)}
            </span>
          </div>
          <Progress percent={diskUsage} status={diskUsage > 90 ? 'exception' : 'normal'} />
        </div>
      </div>
    </Card>
  );
};

// 性能指标
const MetricsCard = () => {
  const { data: metrics } = useSWR<Dashboard.SystemMetricsData>(dashboardAPI.metricsUrl);
  return (
    <Card
      title="7 days performance metrics"
      rootClassName="flex flex-col"
      classNames={{
        body: 'flex-1 place-content-center',
      }}
    >
      <div className="grid grid-cols-3 gap-4">
        <div className="text-center">
          <div className="text-2xl font-bold text-blue-600">{metrics?.avgResponseTime}ms</div>
          <div className="text-gray-500">Average response time</div>
        </div>
        <div className="text-center">
          <div className="text-2xl font-bold text-green-600">{metrics?.errorRate?.toFixed(1)}%</div>
          <div className="text-gray-500">Error rate</div>
        </div>
        <div className="text-center">
          <div className="text-2xl font-bold text-purple-600">{metrics?.totalRequests} times</div>
          <div className="text-gray-500">Total requests</div>
        </div>
      </div>
    </Card>
  );
};

// 用户活动趋势
const UserActivityTrendCard = () => {
  const { data } = useSWR<Dashboard.UserActivityChart>(dashboardAPI.trendsUrl);
  return (
    <div className="grid grid-cols-2 gap-4">
      <Card title="30 days user login trend">
        <Line
          data={data?.dailyLogins || []}
          xField="date"
          yField="count"
          height={300}
          axis={{
            y: {
              labelFormatter: (v: number) => Math.round(v),
            },
          }}
        />
      </Card>
      <Card title="24 hours active users">
        <Column
          data={data?.hourlyActive || []}
          xField="date"
          yField="count"
          height={300}
          axis={{
            y: {
              labelFormatter: (v: number) => Math.round(v),
            },
          }}
          style={{
            radiusTopLeft: 10,
            radiusTopRight: 10,
          }}
        />
      </Card>
    </div>
  );
};
