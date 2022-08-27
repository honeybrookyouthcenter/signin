using Windows.UI;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Media;

namespace HBCCSignIn.Controls
{
    public sealed partial class BigIconButton : UserControl
    {
        public BigIconButton()
        {
            InitializeComponent();
            Background = new SolidColorBrush(Colors.Transparent);
        }

        /// <summary>
        /// The text to show in the icon area
        /// </summary>
        public string Icon
        {
            get { return (string)GetValue(IconProperty); }
            set { SetValue(IconProperty, value); }
        }
        public static readonly DependencyProperty IconProperty =
            DependencyProperty.Register("Icon", typeof(string), typeof(BigIconButton), new PropertyMetadata(""));

        /// <summary>
        /// The color of the icon
        /// </summary>
        public Brush IconForeground
        {
            get { return (Brush)GetValue(IconForegroundProperty); }
            set { SetValue(IconForegroundProperty, value); }
        }
        public static readonly DependencyProperty IconForegroundProperty =
            DependencyProperty.Register("IconForeground", typeof(Brush), typeof(BigIconButton), new PropertyMetadata(new SolidColorBrush(Colors.Black)));

        /// <summary>
        /// The text to show in the label area
        /// </summary>
        public string Label
        {
            get { return (string)GetValue(LabelProperty); }
            set { SetValue(LabelProperty, value); }
        }
        public static readonly DependencyProperty LabelProperty =
            DependencyProperty.Register("Label", typeof(string), typeof(BigIconButton), new PropertyMetadata("Click me"));
    }
}
