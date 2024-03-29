﻿using SignIn.Logic.Data;
using SignIn.Uwp.Data;
using System;
using System.ComponentModel;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices.WindowsRuntime;
using System.Threading.Tasks;
using Windows.Graphics.Display;
using Windows.Graphics.Imaging;
using Windows.Storage;
using Windows.UI.Core;
using Windows.UI.Input.Inking;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Input;
using Windows.UI.Xaml.Media.Imaging;

namespace SignIn.Uwp.Controls
{
    public sealed partial class AgreementControl : StackPanel, INotifyPropertyChanged
    {
        string Agreement => $@"By signing this agreement, I voluntarily agree to assume all of the foregoing risks and accept sole responsibility for any injury to my child(ren) or myself (including, but not limited to, personal injury, disability, and death), illness, damage, loss, claim, liability, or expense, of any kind, that I or my child(ren) may experience or incur in connection with my child(ren)’s attendance at {DataProvider.Current.AppName} or participation in {DataProvider.Current.AppName} programming (“Claims”). On my behalf, and on behalf of my children, I hereby release, covenant not to sue, discharge, and hold harmless {DataProvider.Current.AppName}, its employees, agents, and representatives, of and from the Claims, including all liabilities, claims, actions, damages, costs or expenses of any kind arising out of or relating thereto. I understand and agree that this release includes any Claims based on the actions, omissions, or negligence of {DataProvider.Current.AppName}, its employees, agents, and representatives, occurring before, during, or after participation in any of {DataProvider.Current.AppName} programs.";

        public bool IsAgreed => (uiCheck.IsChecked ?? false) && uiCanvas.InkPresenter.StrokeContainer.GetStrokes().Any();

        public AgreementControl()
        {
            InitializeComponent();

            uiCanvas.InkPresenter.InputDeviceTypes = CoreInputDeviceTypes.Mouse | CoreInputDeviceTypes.Pen | CoreInputDeviceTypes.Touch;
            uiCanvas.InkPresenter.StrokesCollected += InkPresenter_StrokesCollected;
        }

        void InkPresenter_StrokesCollected(InkPresenter sender, InkStrokesCollectedEventArgs args) => Update();

        void Clear_Tapped(object sender, TappedRoutedEventArgs e)
        {
            uiCanvas.InkPresenter.StrokeContainer.Clear();
            Update();
        }

        void Terms_Click(Windows.UI.Xaml.Documents.Hyperlink sender, Windows.UI.Xaml.Documents.HyperlinkClickEventArgs args)
        {
            var dialog = new ContentDialog()
            {
                CloseButtonCommand = new StandardUICommand(StandardUICommandKind.Close),
                Content = new ScrollViewer() { Content = new TextBlock() { Text = Agreement, TextWrapping = TextWrapping.WrapWholeWords } }
            };
            _ = dialog.ShowAsync();
        }

        void uiCheck_Checked(object sender, RoutedEventArgs e) => Update();

        void Update() => OnPropertyChanged(nameof(IsAgreed));

        #region Save

        public async Task Save(Person person)
        {
            try
            {
                var folder = await ((UwpDataProvider)DataProvider.Current).GetRootFolder();
                if (folder == null)
                    throw new Exception("Could not get access to the folder to save the signature! The signature was not saved.");

                var covidFolder = await folder.CreateFolderAsync("COVID-19", CreationCollisionOption.OpenIfExists);
                var fileName = GetFileName(person.FullName);
                var file = await covidFolder.CreateFileAsync(fileName, CreationCollisionOption.ReplaceExisting);
                var bitmap = await GetBitmap();
                await SaveToFile(bitmap, file);
            }
            catch (Exception ex)
            {
                await DataProvider.Current.ShowMessage("Could not save the COVID-19 signature, please talk to a staff member.", ex);
            }
        }

        async Task<RenderTargetBitmap> GetBitmap()
        {
            var renderTargetBitmap = new RenderTargetBitmap();
            await renderTargetBitmap.RenderAsync(uiSignature);
            return renderTargetBitmap;
        }

        string GetFileName(string name)
        {
            foreach (char c in Path.GetInvalidFileNameChars())
                name = name.Replace(c, '_');

            return name + ".png";
        }

        async Task SaveToFile(RenderTargetBitmap bitmap, StorageFile file)
        {
            var pixelBuffer = await bitmap.GetPixelsAsync();
            var pixels = pixelBuffer.ToArray();
            var displayInformation = DisplayInformation.GetForCurrentView();
            using (var stream = await file.OpenAsync(FileAccessMode.ReadWrite))
            {
                var encoder = await BitmapEncoder.CreateAsync(BitmapEncoder.PngEncoderId, stream);
                encoder.SetPixelData(BitmapPixelFormat.Bgra8, BitmapAlphaMode.Premultiplied, (uint)bitmap.PixelWidth, (uint)bitmap.PixelHeight, displayInformation.RawDpiX, displayInformation.RawDpiY, pixels);
                await encoder.FlushAsync();
            }
        }

        #endregion

        #region Notify

        /// <summary>
        /// Property Changed event
        /// </summary>
        public event PropertyChangedEventHandler PropertyChanged;

        /// <summary>
        /// Fire the PropertyChanged event
        /// </summary>
        /// <param name="propertyName">Name of the property that changed (defaults from CallerMemberName)</param>
        void OnPropertyChanged([System.Runtime.CompilerServices.CallerMemberName] string propertyName = "")
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }

        #endregion
    }
}
