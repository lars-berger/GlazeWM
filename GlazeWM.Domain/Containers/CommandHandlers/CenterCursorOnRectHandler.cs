using System.Diagnostics;
using System.Net.NetworkInformation;
using System.Text;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure.Bussing;
using ManagedNativeWifi;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;
using PInvoke;
using static PInvoke.IPHlpApi;
using System;
using System.Linq;
using static Vanara.PInvoke.WlanApi;
using static Vanara.PInvoke.IpHlpApi;
using static Vanara.PInvoke.Ws2_32;
using Vanara.Extensions;
using Vanara.InteropServices;
using System.Runtime.InteropServices;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal sealed class CenterCursorOnRectHandler : ICommandHandler<CenterCursorOnRectCommand>
  {
    private readonly UserConfigService _userConfigService;

    public CenterCursorOnRectHandler(UserConfigService userConfigService)
    {
      _userConfigService = userConfigService;
    }

    public CommandResponse Handle(CenterCursorOnRectCommand command)
    {
      var isEnabled = _userConfigService.GeneralConfig.CursorFollowsFocus;

      if (!isEnabled)
        return CommandResponse.Ok;

      var targetRect = command.TargetRect;

      // Calculate center point of focused window.
      var centerX = targetRect.X + (targetRect.Width / 2);
      var centerY = targetRect.Y + (targetRect.Height / 2);

      SetCursorPos(centerX, centerY);


      // "40:E1:E4:63:72:C5"



      var dwDestAddr = BitConverter.ToUInt32(Encoding.ASCII.GetBytes("8.8.8.8"));
      GetBestInterface(dwDestAddr, out var dwBestIfIndex);
      var primaryAdapter = GetAdaptersAddresses(GetAdaptersAddressesFlags.GAA_FLAG_INCLUDE_GATEWAYS).FirstOrDefault(
          r => r.OperStatus == IF_OPER_STATUS.IfOperStatusUp
          && r.TunnelType == TUNNEL_TYPE.TUNNEL_TYPE_NONE
          && r.FirstGatewayAddress != IntPtr.Zero
          && r.IfIndex == dwBestIfIndex
        );

      switch (primaryAdapter.IfType)
      {
        case IFTYPE.IF_TYPE_ETHERNET_CSMACD:
        case IFTYPE.IF_TYPE_ETHERNET_3MBIT:
          Debug.WriteLine("HEREEE");
          break;
        case IFTYPE.IF_TYPE_IEEE80211:
          var hWlan = WlanOpenHandle();

          WlanEnumInterfaces(hWlan, default, out var list).ThrowIfFailed();
          if (list.dwNumberOfItems < 1)
            throw new InvalidOperationException("No WLAN interfaces.");
          var intf = list.InterfaceInfo[0].InterfaceGuid;

          var getType = CorrespondingTypeAttribute.GetCorrespondingTypes(WLAN_INTF_OPCODE.wlan_intf_opcode_current_connection, CorrespondingAction.Get).FirstOrDefault();
          var ee = WlanQueryInterface(hWlan, intf, WLAN_INTF_OPCODE.wlan_intf_opcode_current_connection, default, out var sz, out var data, out var type);
          if (ee.Failed)
            break;
          var yyy = (WLAN_CONNECTION_ATTRIBUTES)data.DangerousGetHandle().Convert(sz, getType);
          var sigQual = yyy.wlanAssociationAttributes.wlanSignalQuality;
          break;
      }

      bool pingable = false;
      Ping pinger = null;
      try
      {
        pinger = new Ping();
        PingReply reply = pinger.Send("8.8.8.8");
        pingable = reply.Status == IPStatus.Success;
      }
      catch (PingException)
      {
        // Discard PingExceptions and return false;
      }
      finally
      {
        if (pinger != null)
        {
          pinger.Dispose();
        }
      }








      // var bssNetworks = NativeWifi.EnumerateBssNetworks();
      // var allC = NativeWifi.EnumerateConnectedNetworkSsids();
      // var connectedSSID = NativeWifi.EnumerateConnectedNetworkSsids().FirstOrDefault();
      // foreach (var nw in bssNetworks)
      // {
      //   Debug.WriteLine(nw.Ssid.ToString() == connectedSSID.ToString());
      // }
      // var speed = bssNetworks.Where(network => network.Ssid.ToString() == connectedSSID.ToString()).FirstOrDefault().LinkQuality;
      // var connectedInterface = NativeWifi.EnumerateInterfaceConnections();




      // // {1b243423-099e-423f-8500-e5785e026467}
      // var primaryAdapter = GetAdaptersAddresses(GetAdaptersAddressesFlags.GAA_FLAG_INCLUDE_GATEWAYS).FirstOrDefault(r => r.OperStatus == IF_OPER_STATUS.IfOperStatusUp && r.TunnelType == TUNNEL_TYPE.TUNNEL_TYPE_NONE && r.FirstGatewayAddress != IntPtr.Zero);
      // var primaryIndex = primaryAdapter.IfIndex;





      // uint alen = 15000;
      // var amem = new SafeCoTaskMemHandle((int)alen);
      // var ai = GetAdaptersInfo((IntPtr)amem, ref alen);
      // // AdapterName [string]:
      // // "{1B243423-099E-423F-8500-E5785E026467}"
      // var az = ((IntPtr)amem).LinkedListToIEnum<IP_ADAPTER_INFO>(i => i.Next);

      // WlanEnumInterfaces(hWlan, default, out var list).ThrowIfFailed();
      // if (list.dwNumberOfItems < 1)
      //   throw new InvalidOperationException("No WLAN interfaces.");
      // var intf = list.InterfaceInfo[0].InterfaceGuid;
      // var conn = list.InterfaceInfo[0].isState == WLAN_INTERFACE_STATE.wlan_interface_state_connected;
      // Debug.WriteLine(intf.ToString());

      // var q = WlanGetNetworkBssList(hWlan, intf, IntPtr.Zero, DOT11_BSS_TYPE.dot11_BSS_type_any, true, default, out var mem);
      // var elist = mem.DangerousGetHandle().ToStructure<WLAN_BSS_LIST>();
      // // {40:E1:E4:63:72:C5}
      // var x = WlanGetAvailableNetworkList(hWlan, intf, 3, default, out var listz);
      // var z = WlanGetInterfaceCapability(hWlan, intf, default, out var listzz);



      // WLAN_PROFILE_FLAGS flags = 0;
      // WlanGetProfileList(hWlan, intf, default, out var qeflist).ThrowIfFailed();
      // WlanGetProfile(hWlan, intf, qeflist.ProfileInfo[0].strProfileName, default, out var xml, ref flags, out var access);




      // MIB_IPADDRTABLE t = GetIpAddrTable();

      // uint len = 15000;
      // var memm = new SafeCoTaskMemHandle((int)len);
      // var xy = GetPerAdapterInfo(primaryAdapter.IfIndex);


      // // Find a matching .NET interface object with the given index.
      // foreach (var networkInterface in NetworkInterface.GetAllNetworkInterfaces())
      //   if (networkInterface.GetIPProperties().GetIPv4Properties().Index == dwBestIfIndex)
      //   {
      //     // "{1B243423-099E-423F-8500-E5785E026467}"

      //     var y = bssNetworks.Where(x => x.Interface.Id.ToString() == networkInterface.Id);

      //   }

      return CommandResponse.Ok;
    }
  }
}
