export interface ErrorMessage {
  code: string;
  message: string;
}

export const errorCodeMap: Record<string, string> = {
  // 通用错误
  ERR_NOT_FOUND: '资源不存在',
  ERR_UNAUTHORIZED: '未授权访问，请登录',
  ERR_FORBIDDEN: '无权限访问',
  ERR_BAD_REQUEST: '请求参数错误',
  ERR_DATABASE: '数据库错误',
  ERR_INTERNAL: '服务器内部错误',

  // 用户认证错误
  ERR_INVALID_CREDENTIALS: '用户名或密码错误',
  ERR_EMPTY_USERNAME: '用户名不能为空',
  ERR_PASSWORD_TOO_SHORT: '密码长度不能少于 6 位',
  ERR_USER_EXISTS: '该用户名已被注册',

  // 数据引用错误
  ERR_REFERENCE_NOT_FOUND: '引用的数据不存在',

  // 物品相关错误
  ERR_ITEM_NOT_FOUND: '物品不存在',
  ERR_ITEM_NAME_EMPTY: '物品名称不能为空',

  // 分类相关错误
  ERR_CATEGORY_NOT_FOUND: '分类不存在',
  ERR_CATEGORY_NAME_EMPTY: '分类名称不能为空',
  ERR_CATEGORY_EXISTS: '分类名称已存在',

  // 地点相关错误
  ERR_LOCATION_NOT_FOUND: '地点不存在',
  ERR_LOCATION_NAME_EMPTY: '地点名称不能为空',
  ERR_LOCATION_EXISTS: '地点名称已存在',

  // 批次相关错误
  ERR_BATCH_NOT_FOUND: '批次不存在',
  ERR_BATCH_QUANTITY_INVALID: '数量必须大于 0',
};

export function getErrorMessage(code: string): string {
  return errorCodeMap[code] || '未知错误';
}
