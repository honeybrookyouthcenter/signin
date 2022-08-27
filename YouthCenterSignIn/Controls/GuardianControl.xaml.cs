using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using SignIn.Logic.Data;

namespace SignIn.Controls
{
    public sealed partial class GuardianControl : UserControl
    {
        public GuardianControl()
        {
            InitializeComponent();
        }

        public void Focus()
        {
            uiGuardianName.Focus(FocusState.Keyboard);
        }

        public Guardian Guardian { get => (Guardian)GetValue(GuardianProperty); set => SetValue(GuardianProperty, value); }
        public static readonly DependencyProperty GuardianProperty =
            DependencyProperty.Register(nameof(Guardian), typeof(Guardian), typeof(GuardianControl), new PropertyMetadata(default(Guardian)));
    }
}
