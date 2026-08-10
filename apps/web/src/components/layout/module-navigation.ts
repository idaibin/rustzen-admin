/**
 * Keep one sidebar/search item for a route even when two module capabilities
 * expose the same surface. The effective navigation API already filters items
 * by the current user's grants; when both grants are present, the later
 * manifest item wins deterministically (the module manifest sort order is the
 * presentation order). A user with only one grant still receives that item's
 * label and permission.
 */
export function dedupeModuleNavigation(
    navigation: SystemModule.NavigationItem[],
): SystemModule.NavigationItem[] {
    const byRoute = new Map<string, SystemModule.NavigationItem>();
    for (const item of navigation) {
        byRoute.set(`${item.module}:${item.path}`, item);
    }
    return Array.from(byRoute.values());
}
