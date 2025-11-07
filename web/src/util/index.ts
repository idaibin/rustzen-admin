export const calculatePercent = (source?: number, total?: number) => {
  if (!source || !total) return 0;
  const percent = (source / total) * 100;
  return Number(percent.toFixed(1));
};

const UNIT_MAP = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
export const convertUnit = (source?: number) => {
  if (!source) return 0;
  const index = Math.floor(Math.log(source) / Math.log(1024));
  return `${(source / Math.pow(1024, index)).toFixed(1)}${UNIT_MAP[index]}`;
};
