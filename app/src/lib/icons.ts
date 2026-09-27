import add from '@material-symbols/svg-400/outlined/add.svg?raw';
import archive from '@material-symbols/svg-400/outlined/archive.svg?raw';
import block from '@material-symbols/svg-400/outlined/block.svg?raw';
import check from '@material-symbols/svg-400/outlined/check.svg?raw';
import checkCircle from '@material-symbols/svg-400/outlined/check_circle.svg?raw';
import cleaningServices from '@material-symbols/svg-400/outlined/cleaning_services.svg?raw';
import close from '@material-symbols/svg-400/outlined/close.svg?raw';
import cloudDone from '@material-symbols/svg-400/outlined/cloud_done.svg?raw';
import cropSquare from '@material-symbols/svg-400/outlined/crop_square.svg?raw';
import darkMode from '@material-symbols/svg-400/outlined/dark_mode.svg?raw';
import trash from '@material-symbols/svg-400/outlined/delete.svg?raw';
import edit from '@material-symbols/svg-400/outlined/edit.svg?raw';
import filterAlt from '@material-symbols/svg-400/outlined/filter_alt.svg?raw';
import group from '@material-symbols/svg-400/outlined/group.svg?raw';
import home from '@material-symbols/svg-400/outlined/home.svg?raw';
import howToReg from '@material-symbols/svg-400/outlined/how_to_reg.svg?raw';
import inbox from '@material-symbols/svg-400/outlined/inbox.svg?raw';
import info from '@material-symbols/svg-400/outlined/info.svg?raw';
import key from '@material-symbols/svg-400/outlined/key.svg?raw';
import label from '@material-symbols/svg-400/outlined/label.svg?raw';
import language from '@material-symbols/svg-400/outlined/language.svg?raw';
import lightMode from '@material-symbols/svg-400/outlined/light_mode.svg?raw';
import lock from '@material-symbols/svg-400/outlined/lock.svg?raw';
import lockOpen from '@material-symbols/svg-400/outlined/lock_open.svg?raw';
import login from '@material-symbols/svg-400/outlined/login.svg?raw';
import logout from '@material-symbols/svg-400/outlined/logout.svg?raw';
import mail from '@material-symbols/svg-400/outlined/mail.svg?raw';
import markEmailRead from '@material-symbols/svg-400/outlined/mark_email_read.svg?raw';
import menu from '@material-symbols/svg-400/outlined/menu.svg?raw';
import newLabel from '@material-symbols/svg-400/outlined/new_label.svg?raw';
import openInNew from '@material-symbols/svg-400/outlined/open_in_new.svg?raw';
import progressActivity from '@material-symbols/svg-400/outlined/progress_activity.svg?raw';
import psychology from '@material-symbols/svg-400/outlined/psychology.svg?raw';
import refresh from '@material-symbols/svg-400/outlined/refresh.svg?raw';
import remove from '@material-symbols/svg-400/outlined/remove.svg?raw';
import report from '@material-symbols/svg-400/outlined/report.svg?raw';
import search from '@material-symbols/svg-400/outlined/search.svg?raw';
import settings from '@material-symbols/svg-400/outlined/settings.svg?raw';
import shoppingBag from '@material-symbols/svg-400/outlined/shopping_bag.svg?raw';
import swapHoriz from '@material-symbols/svg-400/outlined/swap_horiz.svg?raw';
import unsubscribe from '@material-symbols/svg-400/outlined/unsubscribe.svg?raw';
import upgrade from '@material-symbols/svg-400/outlined/upgrade.svg?raw';
import upload from '@material-symbols/svg-400/outlined/upload.svg?raw';
import visibilityOff from '@material-symbols/svg-400/outlined/visibility_off.svg?raw';
import wandStars from '@material-symbols/svg-400/outlined/wand_stars.svg?raw';
import warning from '@material-symbols/svg-400/outlined/warning.svg?raw';
import trashFilled from '@material-symbols/svg-400/outlined/delete-fill.svg?raw';
import homeFilled from '@material-symbols/svg-400/outlined/home-fill.svg?raw';
import labelFilled from '@material-symbols/svg-400/outlined/label-fill.svg?raw';
import settingsFilled from '@material-symbols/svg-400/outlined/settings-fill.svg?raw';
import unsubscribeFilled from '@material-symbols/svg-400/outlined/unsubscribe-fill.svg?raw';
import wandStarsFilled from '@material-symbols/svg-400/outlined/wand_stars-fill.svg?raw';

export const ICONS = {
  add,
  archive,
  block,
  check,
  checkCircle,
  cleaningServices,
  close,
  cloudDone,
  cropSquare,
  darkMode,
  delete: trash,
  edit,
  filterAlt,
  group,
  home,
  howToReg,
  inbox,
  info,
  key,
  label,
  language,
  lightMode,
  lock,
  lockOpen,
  login,
  logout,
  mail,
  markEmailRead,
  menu,
  newLabel,
  openInNew,
  progressActivity,
  psychology,
  refresh,
  remove,
  report,
  search,
  settings,
  shoppingBag,
  swapHoriz,
  unsubscribe,
  upgrade,
  upload,
  visibilityOff,
  wandStars,
  warning,
  deleteFilled: trashFilled,
  homeFilled,
  labelFilled,
  settingsFilled,
  unsubscribeFilled,
  wandStarsFilled,
} as const;

export type IconName = keyof typeof ICONS;
