export {};

declare global {
  interface Window {
    __opengithubApplyTheme?: (theme: string, fontSize: string) => void;
  }
}
