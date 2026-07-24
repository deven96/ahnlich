import type {ReactNode} from 'react';
import {useThemeConfig} from '@docusaurus/theme-common';
import {
  splitNavbarItems,
  useNavbarMobileSidebar,
} from '@docusaurus/theme-common/internal';
import NavbarItem, {type Props as NavbarItemConfig} from '@theme/NavbarItem';
import NavbarColorModeToggle from '@theme/Navbar/ColorModeToggle';
import SearchBar from '@theme/SearchBar';
import NavbarMobileSidebarToggle from '@theme/Navbar/MobileSidebar/Toggle';
import NavbarLogo from '@theme/Navbar/Logo';
import NavbarSearch from '@theme/Navbar/Search';

function useNavbarItems(): NavbarItemConfig[] {
  return useThemeConfig().navbar.items as NavbarItemConfig[];
}

function NavbarItems({items}: {items: NavbarItemConfig[]}): ReactNode {
  return (
    <>
      {items.map((item, i) => (
        <NavbarItem {...item} key={i} />
      ))}
    </>
  );
}

/**
 * Revamped navbar layout: three columns — brand on the left, a centered search
 * bar in the middle, and nav links + theme toggle on the right.
 */
export default function NavbarContent(): ReactNode {
  const mobileSidebar = useNavbarMobileSidebar();
  const items = useNavbarItems();
  const [leftItems, rightItems] = splitNavbarItems(items);
  const searchBarItem = items.find((item) => item.type === 'search');

  return (
    <div className="navbar__inner ahn-navbar-3col">
      <div className="navbar__items">
        {!mobileSidebar.disabled && <NavbarMobileSidebarToggle />}
        <NavbarLogo />
        <NavbarItems items={leftItems} />
      </div>

      <div className="ahn-navbar-center">
        {!searchBarItem && (
          <NavbarSearch>
            <SearchBar />
          </NavbarSearch>
        )}
      </div>

      <div className="navbar__items navbar__items--right">
        <NavbarItems items={rightItems} />
        <NavbarColorModeToggle className="ahn-navbar-color-toggle" />
        {searchBarItem && (
          <NavbarSearch>
            <SearchBar />
          </NavbarSearch>
        )}
      </div>
    </div>
  );
}
