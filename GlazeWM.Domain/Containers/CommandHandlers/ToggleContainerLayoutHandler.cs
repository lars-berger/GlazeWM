using System.Linq;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal sealed class ToggleContainerLayoutHandler : ICommandHandler<ToggleContainerLayoutCommand>
  {
    private readonly Bus _bus;

    public ToggleContainerLayoutHandler(Bus bus)
    {
      _bus = bus;
    }
    public CommandResponse Handle(ToggleContainerLayoutCommand command)
    {
      var container = command.Container;
      var currentLayout = (container.Parent as SplitContainer).Layout;
      var newLayout = currentLayout == Layout.Horizontal ? Layout.Vertical : Layout.Horizontal;
      _bus.Invoke(new ChangeContainerLayoutCommand(container, newLayout));
      return CommandResponse.Ok;
    }
  }
}
