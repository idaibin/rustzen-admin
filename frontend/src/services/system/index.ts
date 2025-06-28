// 系统管理模块统一导出

export { userAPI } from './user';
export { roleAPI } from './role';
export { menuAPI } from './menu';
export { dictAPI } from './dict';
export { logAPI } from './log';

// 默认导出系统管理API集合
import { userAPI } from './user';
import { roleAPI } from './role';
import { menuAPI } from './menu';
import { dictAPI } from './dict';
import { logAPI } from './log';

export default {
  user: userAPI,
  role: roleAPI,
  menu: menuAPI,
  dict: dictAPI,
  log: logAPI,
};
