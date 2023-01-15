using System;
using System.Reactive.Linq;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Windows;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Events;

namespace GlazeWM.Bar.Components
{
  public class WindowTitleComponentViewModel : ComponentViewModel
  {
    private readonly Bus _bus = ServiceLocator.GetRequiredService<Bus>();
    private readonly ContainerService _containerService =
      ServiceLocator.GetRequiredService<ContainerService>();

    public string _focusedWindowTitle = string.Empty;
    public string FocusedWindowTitle
    {
      get => _focusedWindowTitle;
      set
      {
        _focusedWindowTitle = value;
        OnPropertyChanged(nameof(FocusedWindowTitle));
      }
    }

    public WindowTitleComponentViewModel(
      BarViewModel parentViewModel,
      WindowTitleComponentConfig config) : base(parentViewModel, config)
    {
      // TODO: Remove this duplicate code
      _bus.Events.OfType<WindowTitleChangedEvent>()
        .Subscribe(@event =>
        {
          var focusedWindow = _containerService.FocusedContainer as Window;

          if (@event.WindowHandle != focusedWindow.Handle)
            return;

          FocusedWindowTitle = focusedWindow?.Title ?? string.Empty;
        });
      _bus.Events.OfType<WindowFocusedEvent>()
        .Subscribe(@event =>
        {
          var focusedWindow = _containerService.FocusedContainer as Window;

          if (@event.WindowHandle != focusedWindow.Handle)
            return;

          FocusedWindowTitle = focusedWindow?.Title ?? string.Empty;
        });

    }
  }
}